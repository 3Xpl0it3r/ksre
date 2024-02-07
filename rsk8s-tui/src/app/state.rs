use std::collections::HashMap;
use std::sync::Arc;

use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use tokio_util::sync::CancellationToken;

use crate::event::{self, CusKey, Event, KubeEvent};
use crate::kubernetes::api::RtObject;
use crate::kubernetes::indexer::StoreIndex;

use super::action::{Mode, Route};
use super::keybind::{
    HandleFn, KeyBindings, KeyContext, DEFAULT_DPL_KEYBIND, DEFAULT_ERROR_HANDLE,
    DEFAULT_NODE_KEYBIND, DEFAULT_NOP_KEYBINDS, DEFAULT_POD_KEYBIND, KEY_CONTEXT_RECONCILE,
};

impl StatefulList {
    pub fn next(&mut self) {
        if self.index == self.items.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }
    pub fn prev(&mut self) {
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

pub struct Executor {
    pub run_fn: Option<HandleFn>,
    pub stop_fn: CancellationToken,
    state: bool,
}

impl Executor {
    fn new(hander: Option<HandleFn>, is_running: bool) -> Executor {
        Executor {
            run_fn: hander,
            state: false,
            stop_fn: CancellationToken::new(),
        }
    }
}

pub struct AppState {
    pub cur_mode: Mode, //当前模式
    pub input_char: String,
    pub reay: bool,
    // 当前选中的tab页面
    pub cur_route: Route,
    // 当前选中的pod
    pub cur_pod: i32,
    // 当前选中的节点
    pub cur_node: i32,
    // 存储 StoreIndex<Clone, Clone>
    pub store_pods: StoreIndex<PodSpec, PodStatus>,
    // ui list 缓存项目
    pub cache_items: StatefulList,
    // 用来存储临时任务的输出
    pub stdout_buffer: Arc<tokio::sync::RwLock<Vec<String>>>,
    // 记录触发command模式的handler,用于下一次继续触发command
    pub executor: Option<Executor>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            cur_mode: Mode::Normal,
            cache_items: StatefulList {
                items: Vec::new(),
                index: 0,
                refresh: true,
                sub_items: HashMap::new(),
            },
            input_char: String::new(),
            reay: true,
            cur_route: Route::PodIndex,
            cur_pod: 0,
            cur_node: 0,
            store_pods: StoreIndex::new(),
            stdout_buffer: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            executor: None,
        }
    }
}

///
/// kubernetes 事件处理
impl AppState {
    // 同步来自接收apiserver 数据事件
    pub fn handle_pod_reflect_event(&mut self, event: KubeEvent<PodSpec, PodStatus>) -> KeyContext {
        match event {
            KubeEvent::OnAdd(obj) => {
                self.on_add(&obj);
                self.store_pods.add(obj).expect("add object failed");
            }
            KubeEvent::OnDel(obj) => {
                self.on_del(&obj);
                self.store_pods.add(obj).expect("del obj failed");
            }
        }
        DEFAULT_POD_KEYBIND.tick
    }

    fn on_add(&mut self, obj: &RtObject<PodSpec, PodStatus>) {
        self.cache_items.items.push(obj.resource_name());
    }
    fn on_del(&mut self, obj: &RtObject<PodSpec, PodStatus>) {}
}

// AppState[#TODO] (should add some comments)
impl AppState {
    // 事件处理逻辑如下 ：w
    // normal(triggerd)   -----> insert ----(esc) --> normal <-------------  (reloop, without consume esc key)
    //                      /\      |                                      |
    //                      |      -------------------insert -----(esc)---|
    //                      |      |
    //                      |      |________(enter)---command ----(esc)---> normal(reloop, without comsume esc key)
    //                      |                            |
    //                      |-----------------------------(command reabck to insert, should cleanr all buffer in userinput)
    pub fn handle_terminal_key_event(&mut self, event: Event) -> KeyContext {
        if !self.sync_mode_with_continue(event) {
            return KEY_CONTEXT_RECONCILE;
        }
        // 如果当前正在处于insert模式直接处理user insert
        let keybind = self.get_keybings();

        match event {
            Event::Tick => keybind.tick,
            Event::Error => keybind.tick,
            Event::Key(key) => match key {
                CusKey::Tab => keybind.tab,
                CusKey::J => keybind.j,
                CusKey::K => keybind.k,
                CusKey::L => keybind.l, // show log
                //
                CusKey::T => {
                    // 进入terminal 模式
                    self.cur_route = Route::PodTerm;
                    self.cur_mode = Mode::Insert;
                    self.executor = Some(Executor::new(keybind.t.handler, false));
                    KEY_CONTEXT_RECONCILE
                }
                CusKey::Q => keybind.q,
                _ => keybind.tick,
            },
        }
    }

    // 根据当前的route id来获取对应的keybinds
    fn get_keybings(&self) -> KeyBindings {
        if self.cur_route >= Route::PodIndex && self.cur_route <= Route::PodEnd {
            return DEFAULT_POD_KEYBIND;
        }
        if self.cur_route >= Route::DeployIndex && self.cur_route <= Route::DeployEnd {
            return DEFAULT_DPL_KEYBIND;
        }
        if self.cur_route >= Route::NodeIndex && self.cur_route <= Route::NodeEnd {
            return DEFAULT_NODE_KEYBIND;
        }
        DEFAULT_NOP_KEYBINDS
    }

    //模式切换 | ---esc-> continue
    //         |
    // true: current is normal,continue
    // false, comsume esc key, reloop keyevent if current mode is insert or command
    fn sync_mode_with_continue(&mut self, event: Event) -> bool {
        match self.cur_mode {
            Mode::Normal => true,
            Mode::Insert => {
                if let Event::Key(key) = event {
                    match key {
                        CusKey::Esc => {
                            self.cur_mode = Mode::Normal;
                            if let Some(ref ctx) = self.executor {
                                if ctx.stop_fn.is_cancelled() {
                                    ctx.stop_fn.cancel();
                                }
                                self.executor.take();
                            }
                            false
                        }
                        CusKey::Enter => {
                            self.cur_mode = Mode::Command;
                            false
                        }
                        _ => {
                            self.consume_user_input(key);
                            false
                        }
                    }
                } else {
                    false
                }
            }
            Mode::Command => {
                if let Event::Key(key) = event {
                    if let CusKey::Esc = key {
                        self.cur_mode = Mode::Normal;
                        if let Some(ref ctx) = self.executor {
                            if ctx.stop_fn.is_cancelled() {
                                ctx.stop_fn.cancel();
                            }
                            self.executor.take();
                        }
                    }
                    false
                } else {
                    false
                }
            }
        }
    }

    fn consume_user_input(&mut self, key: CusKey) {
        match key {
            CusKey::Backspace => {
                if !self.input_char.is_empty() {
                    self.input_char.pop();
                }
            }
            CusKey::Enter => {
                self.cur_mode = Mode::Command;
            }
            _ => self.input_char.push(key.char()),
        }
    }

    pub fn stdout_buffer_reader(&self) -> Arc<tokio::sync::RwLock<Vec<String>>> {
        self.stdout_buffer.clone()
    }

    pub fn stdout_buffer_writer(&self) -> Arc<tokio::sync::RwLock<Vec<String>>> {
        { /* self.stdout_buffer.try_write().unwrap().clear(); */ }
        self.stdout_buffer.clone()
    }
    pub fn next_route(&mut self) {
        self.cur_route = self.cur_route.next();
    }

    // normal 模式下self.command_ctx是None
    pub fn consume_command_task(&mut self) -> Option<Executor> {
        self.executor.as_ref()?;

        if let Mode::Normal = self.cur_mode {
            return None;
        }
        // 如果是insert
        // 模式直接不做任何处理，同时也拒绝任何handler，直接reconcile重新渲染图形就可以
        if let Mode::Insert = self.cur_mode {
            return Some(Executor {
                run_fn: None,
                state: false,
                stop_fn: CancellationToken::new(),
            });
        }
        self.cur_mode = Mode::Insert;
        Some(Executor::new(self.executor.as_ref().unwrap().run_fn, true))
    }
}
