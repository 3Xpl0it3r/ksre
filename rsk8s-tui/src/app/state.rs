use std::collections::HashMap;
use std::sync::Arc;

use color_eyre::eyre::Result;
use futures::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::{Pod, PodSpec, PodStatus};
use kube::api::LogParams;
use kube::{Api, Client as KubeClient};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::event::{Event, KubeEvent};
use crate::kubernetes::api::RtObject;
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
    pub items: Vec<String>,
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

    pub cache_pods: StoreIndex<PodSpec, PodStatus>,

    pub select_items: StatefulList,

    // 用来存储临时任务的输出
    pub stdout_buffer: Arc<tokio::sync::RwLock<Vec<String>>>,
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
            cache_pods: StoreIndex::new(),
            stdout_buffer: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

// AppState[#TODO] (should add some comments)
impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub fn resync_state(&mut self) {}

    fn on_add(&mut self, obj: &RtObject<PodSpec, PodStatus>) {
        // shoud sync with selected items?
    }
    fn on_del(&mut self, obj: &RtObject<PodSpec, PodStatus>) {
        // should sync with seleted items?
    }

    pub fn handle_key_event(&mut self, event: Event) {}

    pub fn resync_cache(&mut self, event: KubeEvent<PodSpec, PodStatus>) {
        match event {
            KubeEvent::OnAdd(obj) => {
                self.on_add(&obj);
                self.cache_pods.add(obj).expect("add object failed");
            }
            KubeEvent::OnDel(obj) => {
                self.on_del(&obj);
                self.cache_pods.add(obj).expect("del obj failed");
            }
        }
    }

    pub fn empty_stdout_buffer(&mut self) {
        self.stdout_buffer.try_write().unwrap().clear();
    }
}
