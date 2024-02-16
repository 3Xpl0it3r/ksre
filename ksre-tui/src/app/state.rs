use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher};
use tokio_util::sync::CancellationToken;
use tui_textarea::TextArea;

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
        if self.items.is_empty() {
            return;
        }
        if self.index == self.items.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }
    pub fn prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.index == 0 {
            self.index = self.items.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}

#[derive(Default)]
pub struct StatefulList {
    pub items: Vec<Rc<str>>,
    pub index: usize,
    pub fixed: bool,
}

pub struct Executor {
    pub run_fn: Option<HandleFn>,
    pub stop_fn: CancellationToken,
    state: bool,
    pub prev_route: Route,
}

impl Executor {
    fn new(hander: Option<HandleFn>, is_running: bool, prev_route: Route) -> Executor {
        Executor {
            run_fn: hander,
            state: false,
            stop_fn: CancellationToken::new(),
            prev_route,
        }
    }
}

pub struct AppState {
    pub cur_mode: Mode, //当前模式
    pub input_char: String,
    pub fuzz_matcher: Matcher,
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
    pub namespace_items: StatefulList,
    pub nodes_items: StatefulList,
    // 用来存储临时任务的输出
    pub stdout_buffer: Arc<tokio::sync::RwLock<TextArea<'static>>>,
    // 记录触发command模式的handler,用于下一次继续触发command
    pub executor: Option<Executor>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            cur_mode: Mode::Normal,
            cache_items: StatefulList::default(),
            namespace_items: StatefulList::default(),
            nodes_items: StatefulList::default(),
            input_char: String::new(),
            fuzz_matcher: Matcher::new(Config::DEFAULT),
            reay: true,
            cur_route: Route::PodIndex,
            cur_pod: 0,
            cur_node: 0,
            store_pods: StoreIndex::new(),
            stdout_buffer: Arc::new(tokio::sync::RwLock::new(TextArea::default())),
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
        /* self.cache_items.items.push(obj.resource_name()); */
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
        self.resync_cache_items();
        // 如果当前正在处于insert模式直接处理user insert
        let keybind = self.get_keybings();

        match event {
            Event::Tick => keybind.tick,
            Event::Error => keybind.tick,
            Event::Key(key) => match key {
                CusKey::Esc => {
                    self.handle_esc_key();
                    KEY_CONTEXT_RECONCILE
                }
                CusKey::Enter => {
                    self.handle_enter_key();
                    KEY_CONTEXT_RECONCILE
                }
                CusKey::Tab => keybind.tab,
                CusKey::N => {
                    self.cur_route = Route::PodNamespace;
                    self.namespace_items.fixed = false;
                    KEY_CONTEXT_RECONCILE
                }
                CusKey::J => keybind.j,
                CusKey::K => keybind.k,
                CusKey::L => {
                    self.executor = Some(Executor::new(keybind.l.handler, false, self.cur_route));
                    self.cur_route = Route::PodLog;
                    KEY_CONTEXT_RECONCILE
                } // show log
                CusKey::F => {
                    self.cur_route = Route::PodList;
                    self.cur_mode = Mode::Insert;
                    KEY_CONTEXT_RECONCILE
                } // 检索podlist 赛选
                CusKey::T => {
                    // 进入terminal 模式
                    self.executor = Some(Executor::new(keybind.t.handler, false, self.cur_route));
                    self.cur_route = Route::PodTerm;
                    self.cur_mode = Mode::Insert;
                    KEY_CONTEXT_RECONCILE
                }
                CusKey::Q => keybind.q,
                CusKey::Enter => {
                    if let Route::PodNamespace = self.cur_route {
                        self.namespace_items.fixed = true;
                        self.cur_route = Route::PodIndex;
                    }
                    KEY_CONTEXT_RECONCILE
                }
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

    fn route_reset(&mut self) {
        match self.cur_route {
            m if m < Route::PodEnd => self.cur_route = Route::PodIndex,
            m if m < Route::DeployEnd => self.cur_route = Route::DeployIndex,
            m if m < Route::NodeEnd => self.cur_route = Route::PodIndex,
            _ => self.cur_route = Route::PodIndex,
        }
    }

    // esc key handler
    fn handle_esc_key(&mut self) {
        self.route_reset();
        if let Some(executor) = self.executor.take() {
            executor.stop_fn.cancel();
        }
        // if currnet mode is insert mode, then empty input buffer;
        if let Mode::Insert = self.cur_mode {
            if !self.input_char.is_empty() {
                self.input_char.clear();
            }
        }
        if !self.namespace_items.fixed {
            self.namespace_items.fixed = true;
        }
        // in esc mode, all mode will convert to normal mode
        self.cur_mode = Mode::Normal;
    }

    fn handle_enter_key(&mut self) {
        match self.cur_mode {
            // normal模式下enter键1. select,
            Mode::Normal => {
                if !self.namespace_items.fixed {
                    self.route_reset();
                    self.namespace_items.fixed = true;
                }
            }
            Mode::Insert => {
                self.cur_mode = Mode::Normal;
            }
            Mode::Command => {}
        }
        // insert模式下 ,只需要处理inputchar
        // command模式下,只需要处理command
    }

    fn handle_user_input(&mut self, key: CusKey) {
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

    fn resync_cache_items(&mut self) {
        let namesapce = self
            .namespace_items
            .items
            .get(self.namespace_items.index)
            .unwrap();
        let items = self.store_pods.all_keys(namesapce);
        if self.cur_route as i32 == Route::PodList as i32 && !self.input_char.is_empty() {
            let filter_items = Atom::new(
                self.input_char.as_str(),
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
                false,
            )
            .match_list(items, &mut self.fuzz_matcher)
            .into_iter()
            .map(|x| x.0)
            .collect::<Vec<Rc<str>>>();
            self.cache_items.items = filter_items;
            self.cache_items.index = 0;
        } else {
            self.cache_items.items = items;
        }
    }

    pub fn stdout_buffer_reader(&self) -> Arc<tokio::sync::RwLock<TextArea<'static>>> {
        self.stdout_buffer.clone()
    }

    pub fn stdout_buffer_writer(&self) -> Arc<tokio::sync::RwLock<TextArea<'static>>> {
        { /* self.stdout_buffer.try_write().unwrap().clear(); */ }
        self.stdout_buffer.clone()
    }
    pub fn next_route(&mut self) {
        self.cur_route = self.cur_route.next();
    }

    // normal 模式下self.command_ctx是None
    pub fn consume_command_task(&self) -> Option<&Executor> {
        self.executor.as_ref()?;
        return self.executor.as_ref();
    }

    pub fn get_namespaced_pod(&self) -> Option<(&str, &str)> {
        let namespace = self.namespace_items.items.get(self.namespace_items.index);
        if let Some(pod) = self.cache_items.items.get(self.cache_items.index) {
            Some((namespace.as_ref().unwrap(), pod.as_ref()))
        } else {
            None
        }
    }

    pub fn cancel_executor(&mut self) {
        if let Some(executor) = self.executor.take() {
            executor.stop_fn.cancel();
        }
    }
}
