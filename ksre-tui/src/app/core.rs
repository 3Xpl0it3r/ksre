use std::ops::Sub;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{Namespace, PodSpec, PodStatus};
use kube::{api::ListParams, Api, Client as KubeClient, ResourceExt};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::event::KubeEvent;
use crate::kubernetes::PodMetricsApi;
use crate::tui::Tui;

use super::action::Route;
use super::job::tail_logs;
use super::keybind::Handler;
use super::ui::home::ui_main;

use crate::app::job::{pod_exec, PodExecArgs};
use crate::app::AppState;

pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    should_quit: bool,
    app_state: AppState,
    task0: JoinHandle<()>,
    task1: JoinHandle<()>,
    ready: bool,

    cmd_input_writer: Option<mpsc::Sender<String>>,

    // handler
    // used for fetch pod metrics from kube-metrics-apiserver
    ticker: Ticker,
    pod_metrics_api: PodMetricsApi,
}

impl App {
    pub fn new(
        tui: Tui,
        kube_event: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
        kube_client: KubeClient,
    ) -> Self {
        let pod_metrics_api = PodMetricsApi::new(kube_client.clone());
        Self {
            tui,
            pod_event_rx: kube_event,
            app_state: AppState::new(),
            kube_client,
            should_quit: false,
            task0: tokio::spawn(async {}),
            task1: tokio::spawn(async {}),
            ready: true,
            cmd_input_writer: None,
            ticker: Ticker::new(),
            pod_metrics_api,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let namespace: Api<Namespace> = Api::all(self.kube_client.clone());

        for ns in namespace.list(&ListParams::default()).await.unwrap() {
            self.app_state
                .namespace_items
                .items
                .push(Rc::from(ns.name_any().as_str()));
        }
        let mut keyctx = None;
        loop {
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        keyctx = self.app_state.handle_key_event(event);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if let Ok(event) = kube_event{
                        keyctx = self.app_state.handle_pod_reflect_event(event);
                    }
                },
            }
            // 优先判断是否有cmdcontext , 存在command contxt 意味着当前正在处理command 模式
            if let Some(cmd_context) = self.app_state.consume_command_task() {
                if let Some(handler) = cmd_context.run_fn {
                    handler(self, Some(cmd_context.stop_fn.clone()));
                }
            } else if let Some(_handle) = keyctx.take() {
                _handle(self, None);
                self.ready = true;
            }

            if self.ticker.next() && self.app_state.show_handle_pod_metrics() {
                if let Some((namespace, pod_name)) = self.app_state.pod_name() {
                    let metric = self
                        .pod_metrics_api
                        .get(namespace.as_ref(), pod_name.as_ref())
                        .await
                        .unwrap();
                    self.app_state.handle_pod_metrics(metric);
                }
            }
            self.draw_ui().await;

            if self.should_quit {
                break;
            }
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

// all handler
impl App {
    pub fn handle_next_route(&mut self, _cancel: Option<CancellationToken>) {
        self.app_state.next_route();
    }

    pub fn fake_handlefunction(&mut self, _cancel: Option<CancellationToken>) {}

    pub fn handle_quit(&mut self, _cancel: Option<CancellationToken>) {
        self.should_quit = true;
    }

    pub fn select_items_next(&mut self, _cancel: Option<CancellationToken>) {
        if let Route::PodNamespace = self.app_state.cur_route {
            self.app_state.namespace_items.next();
            return;
        }

        self.app_state.cache_items.next();
    }
    pub fn select_items_prev(&mut self, _cancel: Option<CancellationToken>) {
        if let Route::PodNamespace = self.app_state.cur_route {
            self.app_state.namespace_items.prev();
            return;
        }
        self.app_state.cache_items.prev();
    }

    pub fn handle_pod_logs(&mut self, cancel: Option<CancellationToken>) {
        if self.ready {
            if !self.task0.is_finished() {
                self.task0.abort();
            }
            if !self.task1.is_finished() {
                self.task1.abort();
            }
            let namespace_pod = self.app_state.get_namespaced_pod();
            if namespace_pod.is_none() {
                self.app_state.cancel_executor();
                return;
            }
            self.ready = false;
            let (namespace, pod) = namespace_pod.unwrap();

            let cancel = cancel.unwrap();
            let (log_writer_tx, mut log_reader_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) =
                mpsc::channel(10);
            self.task0 = tokio::spawn(tail_logs(
                cancel,
                self.kube_client.clone(),
                log_writer_tx,
                pod.to_string(),
                namespace.to_string(),
            ));
            let writer = self.app_state.stdout_buffer_writer();
            self.task1 = tokio::spawn(async move {
                {
                    writer.write().await.select_all();
                    writer.write().await.cut();
                }
                while let Some(line) = log_reader_rx.recv().await {
                    writer.write().await.insert_str(line.as_str());
                    writer.write().await.insert_newline();
                }
            })
        }
    }

    pub fn handle_pod_exec(&mut self, cancel: Option<CancellationToken>) {
        if self.ready {
            self.ready = false;
            let cancel = cancel.unwrap();
            let (input_writer, input_reader): (mpsc::Sender<String>, mpsc::Receiver<String>) =
                mpsc::channel(10);
            self.cmd_input_writer = Some(input_writer);
            let pod_args = PodExecArgs {
                kube_client: self.kube_client.clone(),
                pod_name: "default:nginx".to_string(),
                container: None,
            };
            let writer = self.app_state.stdout_buffer_writer();
            self.task1 = tokio::spawn(pod_exec(cancel, writer, input_reader, pod_args));
        } else {
            let writer = self.cmd_input_writer.as_ref().unwrap().clone();
            let input = self.app_state.user_input.clone();
            tokio::spawn(async move {
                writer.send(input).await.expect("send failed");
            });
            self.app_state.user_input.clear();
        }
    }
}

// app tempoary task relative
impl App {}

impl Drop for App {
    fn drop(&mut self) {
        self.should_quit = true;
    }
}

impl Drop for AppState {
    fn drop(&mut self) {}
}

struct Ticker {
    update_at: DateTime<Utc>,
    curr_time: DateTime<Utc>,
}

// Name[#TODO] (should add some comments)
impl Ticker {
    fn new() -> Self {
        Self {
            update_at: chrono::Utc::now(),
            curr_time: chrono::Utc::now(),
        }
    }

    fn next(&mut self) -> bool {
        let current = self.curr_time;
        self.curr_time = chrono::Utc::now();

        if current.sub(self.update_at).num_seconds() > 1 {
            self.update_at = chrono::Utc::now();
            return true;
        }
        false
    }
}
