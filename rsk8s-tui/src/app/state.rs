use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use kube::Client as KubeClient;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::app::action::get_action;
use crate::event::KubeEvent;
use crate::kubernetes::indexer::StoreIndex;
use crate::tui::Tui;

use super::action::{Action, Mode, RouteId};
use super::job::tail_logs;
use super::ui::home::ui_main;

impl StatefulList {
    fn next(&mut self) {
        if self.index == self.items.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }
    fn prev(&mut self) {
        if self.index == 0 {
            self.index = self.items.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}

pub struct StatefulList {
    pub items: Vec<Rc<String>>,
    pub index: usize,
    pub refresh: bool,
    pub sub_items: HashMap<usize, Vec<String>>,
}

pub struct AppState {
    pub op_mode: Mode,
    pub input_char: String,
    pub reay: bool,
    // 当前选中的tab页面
    pub id_cur_route: RouteId,
    // 当前选中的pod
    pub id_cur_pod: i32,
    // 当前选中的节点
    pub id_cur_node: i32,

    pub pod_cache: StoreIndex<PodSpec, PodStatus>,

    pub select_items: StatefulList,

    // 用来存储临时任务的输出
    pub buffer_cache: Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            op_mode: Mode::Normal,
            select_items: StatefulList {
                items: Vec::new(),
                index: 0,
                refresh: true,
                sub_items: HashMap::new(),
            },
            input_char: String::new(),
            reay: true,
            id_cur_route: RouteId::PodIndex,
            id_cur_pod: 0,
            id_cur_node: 0,
            pod_cache: StoreIndex::new(),
            buffer_cache: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    should_quit: bool,
    /* state: Arc<tokio::sync::RwLock<AppState>>, // may be async?  */
    state: AppState,

    ptr_once_ajob: JoinHandle<()>,
    ptr_cron_ajob: JoinHandle<()>,
    ptr_cancel: CancellationToken,
}

impl App {
    pub fn new(
        tui: Tui,
        kube_event: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
        kube_client: KubeClient,
    ) -> Self {
        Self {
            tui,
            pod_event_rx: kube_event,
            kube_client,
            should_quit: false,
            state: AppState::default(),
            ptr_once_ajob: tokio::spawn(async {}),
            ptr_cron_ajob: tokio::spawn(async {}),
            ptr_cancel: CancellationToken::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let mut action = Action::NOP;
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        action = get_action(&event, self.state.id_cur_route);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if kube_event.is_ok() {
                        self.update_storage(kube_event.unwrap());
                        if let RouteId::PodIndex = self.state.id_cur_route {
                            self.state.select_items.refresh = true;
                            action = Action::Resync;
                        }
                    }
                },
            }

            if let Action::NOP = action {
                continue;
            }

            self.update_state(action);
            self.draw_ui();

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn update_storage(&mut self, event: KubeEvent<PodSpec, PodStatus>) {
        match event {
            KubeEvent::OnAdd(obj) => {
                self.state.pod_cache.add(obj).unwrap();
            }
            KubeEvent::OnDel(obj) => {
                self.state.pod_cache.delete(obj).unwrap();
            }
        }
    }

    fn update_state(&mut self, action: Action) {
        match self.state.id_cur_route {
            RouteId::PodIndex => {
                // 检测是否需要刷新select items 只有出现如下两种情况才需要刷新
                // 1. 从其他的tab页面进入到Home页面
                // 2. 当前在Home页面, 但是pod数据刷新了
                if self.state.select_items.refresh {
                    self.state.select_items.refresh = false;
                    self.state.select_items.index = 0;
                    let items = self.state.pod_cache.all_keys("all");
                    if items.is_some() {
                        self.state.select_items.items = items.unwrap();
                    }
                }

                match action {
                    Action::TabNext => {
                        self.state.id_cur_route = self.state.id_cur_route.next();
                    }
                    Action::Resync => todo_function(),
                    Action::Tick => todo_function(),
                    Action::StsItemNext => {
                        self.state.select_items.next();
                    }
                    Action::StsItemPrev => {
                        self.state.select_items.prev();
                    }
                    Action::PodExec => {
                        /* self.state.id_cur_route. */
                    }
                    Action::PodLogs => {}
                    Action::Quit => self.should_quit = true,
                    Action::NOP => todo_function(),
                }
            }
            RouteId::DeployIndex => match action {
                Action::TabNext => self.state.id_cur_route = self.state.id_cur_route.next(),
                Action::Resync => todo_function(),
                Action::Tick => todo_function(),
                Action::StsItemNext => todo_function(),
                Action::StsItemPrev => todo_function(),
                Action::PodExec => todo_function(),
                Action::PodLogs => todo_function(),
                Action::Quit => self.should_quit = true,
                Action::NOP => todo_function(),
            },
            RouteId::NodeIndex => match action {
                Action::TabNext => self.state.id_cur_route = self.state.id_cur_route.next(),
                Action::Resync => todo_function(),
                Action::Tick => todo_function(),
                Action::StsItemNext => todo_function(),
                Action::StsItemPrev => todo_function(),
                Action::PodExec => todo_function(),
                Action::PodLogs => todo_function(),
                Action::Quit => self.should_quit = true,
                Action::NOP => todo_function(),
            },
            _ => todo_function()
        }
    }

    fn draw_ui(&mut self) {
        self.tui.draw(|f| ui_main(f, &mut self.state)).unwrap();
    }

    fn start_job(&mut self, arg: String) {
        {
            self.state.buffer_cache.try_write().unwrap().clear();
        }
        let (tx_stdin, mut rx_stdout) = mpsc::channel(1024);
        self.ptr_cancel = CancellationToken::new();
        self.ptr_once_ajob = tokio::spawn(tail_logs(
            self.ptr_cancel.clone(),
            self.kube_client.clone(),
            tx_stdin,
            arg,
        ));
        let cancellation_token = self.ptr_cancel.clone();
        let buffer = self.state.buffer_cache.clone();
        self.ptr_cron_ajob = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    event = rx_stdout.recv() => {
                        if let Some(line) = event {
                            let mut writer = buffer.write().await;
                            writer.push(line);
                        }
                    }
                }
            }
        })
    }
}

fn todo_function() {}

impl Drop for App {
    fn drop(&mut self) {
        self.should_quit = true;
        if !self.ptr_cancel.is_cancelled() {
            self.ptr_cancel.cancel()
        }
        if !self.ptr_cron_ajob.is_finished() {
            self.ptr_cron_ajob.abort();
        }
        if !self.ptr_cron_ajob.is_finished() {
            self.ptr_cron_ajob.abort()
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {}
}
