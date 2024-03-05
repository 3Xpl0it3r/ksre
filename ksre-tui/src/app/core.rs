use std::rc::Rc;

use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{Namespace, PodSpec, PodStatus};
use kube::{api::ListParams, Api, Client as KubeClient, Resource, ResourceExt};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use tokio::sync::broadcast;

use crate::event::{CusKey, Event, KubeEvent};
use crate::kubernetes::{
    api::{object::RtObject, pod::PodDescribe},
    metrics::pod::{MetricClient, PodMetrics},
};
use crate::tui::Tui;

use super::{
    handler::keybind::{DEPLOYMENT_KEYMAPS, NODE_KEYMAPS, POD_KEYMAPS},
    state::{AppState, Executor, Mode, TabPage},
    ui::home::ui_main,
};

pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    app_state: AppState,
    pod_metrics_api: MetricClient,
}

impl App {
    pub fn new(
        tui: Tui,
        kube_event: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
        kube_client: KubeClient,
    ) -> Self {
        let pod_metrics_api = MetricClient::new(kube_client.clone());
        Self {
            tui,
            pod_event_rx: kube_event,
            app_state: AppState::new(kube_client.clone()),
            kube_client,
            pod_metrics_api,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let namespace: Api<Namespace> = Api::all(self.kube_client.clone());

        for ns in namespace.list(&ListParams::default()).await.unwrap() {
            self.app_state
                .namespace_cache
                .push(Rc::from(ns.name_any().as_str()));
        }

        loop {
            let mut executor: Option<&mut Executor> = None;
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        executor = self.dispatch_tui_keyevents(event);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if let Ok(event) = kube_event{
                        executor = self.dispatch_pod_events(event);
                    }
                },
            }

            if let Some(executor) = executor.take() {
                executor.execute();
            }

            if self.app_state.should_quit() {
                break;
            }

            self.draw_ui().await;
        }

        Ok(())
    }

    async fn draw_ui(&mut self) {
        let stdout_buffer = self.app_state.stdout_buffer.clone();
        let reader = stdout_buffer.read().await;
        self.tui
            .draw(|f| ui_main(f, &mut self.app_state, reader))
            .unwrap();
    }
}

impl App {
    fn dispatch_pod_events(
        &mut self,
        event: KubeEvent<PodSpec, PodStatus>,
    ) -> Option<&mut Executor> {
        match event {
            KubeEvent::OnAdd(obj) => {
                self.pod_on_add(obj);
                self.resync_caches();
            }
            KubeEvent::OnDel(obj) => {
                self.pod_on_del(obj);
                self.resync_caches();
            }
        }
        None
    }

    fn dispatch_tui_keyevents(&mut self, event: Event) -> Option<&mut Executor> {
        self.resync_caches();
        match event {
            Event::Key(key_char) => {
                // 优先处理用户输入
                if Mode::Insert as i32 == self.app_state.get_mode() as i32
                    && !self.handle_user_input(key_char)
                {
                    return None;
                }
                // 第二优先级处理tab键
                if let CusKey::Tab = key_char {
                    self.app_state.next_route();
                    return None;
                }
                // 第三开始dispatch到具体窗口handler来处理对应的keyevent
                let handler = match self.app_state.get_tabpage() {
                    TabPage::Pod => POD_KEYMAPS.get(key_char.as_ref()),
                    TabPage::Deploy => DEPLOYMENT_KEYMAPS.get(key_char.as_ref()),
                    TabPage::Node => NODE_KEYMAPS.get(key_char.as_ref()),
                };

                if let Some(handler) = handler {
                    handler(&mut self.app_state)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    fn handle_user_input(&mut self, key: CusKey) -> bool {
        // true ,input has done
        match key {
            CusKey::Backspace => {
                self.app_state.user_input.pop();
                false
            }
            CusKey::Enter | CusKey::Esc => {
                self.app_state.user_input.complete();
                self.app_state.set_mode(Mode::Normal);
                true
            }
            _ => {
                self.app_state.user_input.push(key.char());
                false
            }
        }
    }
}

impl App {
    fn add_metrics(app_state: &mut AppState, pod_metrics: PodMetrics) {
        if let Some(container_metric) = pod_metrics.containers.into_iter().next() {
            let cpu = container_metric.usage.cpu.0;
            let mem = container_metric.usage.memory.0;
            let metric_line = format!(
                "{:<24}{:<12}{:<12}",
                pod_metrics
                    .metadata
                    .creation_timestamp
                    .unwrap()
                    .0
                    .format("%Y-%m-%d %H:%M:%S"),
                cpu,
                mem,
            );
            app_state.metrics_buffer.insert_str(metric_line);
            app_state.metrics_buffer.insert_newline();
        }
    }
}

impl App {
    fn resync_caches(&mut self) {
        match self.app_state.get_tabpage() {
            TabPage::Pod => self.resync_pod_caches(),
            TabPage::Deploy => self.resync_deployment_caches(),
            TabPage::Node => self.resync_nodes_caches(),
        }
    }
    #[inline]
    fn resync_pod_caches(&mut self) {
        let namespace = self.app_state.namespace_cache.get().unwrap();
        let items = self.app_state.pod_storage.list(namespace.as_ref());
        if !self.app_state.user_input.is_empty() {
            let filter_items = Atom::new(
                self.app_state.user_input.as_str(),
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
                false,
            )
            .match_list(items, &mut self.app_state.fuzz_matcher)
            .into_iter()
            .map(|x| x.0)
            .collect::<Vec<Rc<str>>>();
            self.app_state.cache_items.replace(filter_items);
        } else {
            self.app_state.cache_items.replace(items);
        }

        if !self.app_state.user_input.is_completed() {
            self.app_state.cache_items.reindex();
        }
    }

    #[inline]
    fn resync_deployment_caches(&mut self) {}
    #[inline]
    fn resync_nodes_caches(&mut self) {}
}

// app tempoary task relative
impl App {
    // pod storage onAdd onDel
    #[inline]
    fn pod_on_add(&mut self, obj: RtObject<PodSpec, PodStatus>) {
        let name = obj.0.meta().name.as_deref().unwrap_or_default();
        let namespace = obj.0.meta().namespace.as_deref().unwrap_or_default();
        self.app_state.pod_describes.add(
            namespace.to_string(),
            name.to_string(),
            PodDescribe::from(&obj),
        );
        self.app_state
            .pod_storage
            .add(obj)
            .expect("add object failed");
    }
    #[inline]
    fn pod_on_del(&mut self, obj: RtObject<PodSpec, PodStatus>) {
        let pod_name = obj.0.meta().name.as_deref().unwrap_or_default().to_string();
        let namespace = obj
            .0
            .meta()
            .namespace
            .as_deref()
            .unwrap_or_default()
            .to_string();
        self.app_state
            .pod_storage
            .delete(&namespace, &pod_name)
            .expect("del obj failed");
    }
}

impl Drop for App {
    fn drop(&mut self) {}
}

impl Drop for AppState {
    fn drop(&mut self) {}
}
