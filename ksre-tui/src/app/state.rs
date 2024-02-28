use std::char;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use kube::Resource;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};
use tokio_util::sync::CancellationToken;
use tui_textarea::TextArea;

use super::action::{Mode, Route};
use super::keybind::{
    HandleFn, Handler, KeyBinding, DEFAULT_DEPLOYMENT_KEYBINDING, DEFAULT_NODE_KEYBINDING,
    DEFAULT_POD_KEYBINDING, DEFAULT_NOOP_KEYBINDING,
};
use crate::event::{CusKey, Event, KubeEvent};
use crate::kubernetes::api::metrics::PodMetrics;
use crate::kubernetes::api::PodDescribe;

use crate::kubernetes::{api::RtObject, indexer::StoreIndex};

pub struct AppState {
    pub cur_mode: Mode, //当前模式
    pub user_input: UserInput,
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
    // kubeObj describe信息, Describe是不安全的
    pub kube_obj_describe_cache: KubeDescribeIndices<PodDescribe>,
    // ui list 缓存项目存储kubeObject 列表
    pub cache_items: StatefulList,
    // namespace列表
    pub namespace_items: StatefulList,
    // 节点列表
    pub nodes_items: StatefulList,
    // 用来存储临时任务的输出/ 终端输出
    pub stdout_buffer: Arc<tokio::sync::RwLock<TextArea<'static>>>,
    pub metrics_buffer: TextArea<'static>,
    // 记录触发command模式的handler,用于下一次继续触发command
    pub executor: Option<Executor>,

    // format is timestamp, cpu, memory
    pub pod_metrics_cache: HashMap<String, HashMap<String, CycledCache<(i64, f64, f64)>>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut metrics_buffer = TextArea::default();
        metrics_buffer.set_max_histories(32);
        Self {
            cur_mode: Mode::Normal,
            cache_items: StatefulList::default(),
            namespace_items: StatefulList::default(),
            nodes_items: StatefulList::default(),
            user_input: UserInput::default(),
            fuzz_matcher: Matcher::new(Config::DEFAULT),
            reay: true,
            cur_route: Route::PodIndex,
            cur_pod: 0,
            cur_node: 0,
            store_pods: StoreIndex::new(),
            kube_obj_describe_cache: KubeDescribeIndices::new(),
            stdout_buffer: Arc::new(tokio::sync::RwLock::new(TextArea::default())),
            metrics_buffer,
            executor: None,
            pod_metrics_cache: HashMap::new(),
        }
    }
}

/// kubernetes 事件处理
impl AppState {
    // 同步来自接收apiserver 数据事件
    pub fn handle_pod_reflect_event(&mut self, event: KubeEvent<PodSpec, PodStatus>) -> Handler {
        match event {
            KubeEvent::OnAdd(obj) => {
                self.on_add(obj);
            }
            KubeEvent::OnDel(obj) => {
                self.on_del(obj);
            }
        }
        None
    }

    fn on_add(&mut self, obj: RtObject<PodSpec, PodStatus>) {
        let name = obj.0.meta().name.as_deref().unwrap_or_default();
        let namespace = obj.0.meta().namespace.as_deref().unwrap_or_default();
        self.kube_obj_describe_cache.add(
            namespace.to_string(),
            name.to_string(),
            PodDescribe::from(&obj),
        );
        self.store_pods.add(obj).expect("add object failed");
    }
    fn on_del(&mut self, obj: RtObject<PodSpec, PodStatus>) {
        let _name = obj.0.meta().name.as_deref().unwrap_or_default().to_string();
        let _namespace = obj
            .0
            .meta()
            .namespace
            .as_deref()
            .unwrap_or_default()
            .to_string();
        self.store_pods.delete(obj).expect("del obj failed");
    }
}

// AppState[#TODO] (should add some comments)
impl AppState {
    pub fn handle_key_event(&mut self, event: Event) -> Handler {
        self.resync_cache_items();
        // 如果当前正在处于insert模式直接处理user insert
        let keybind = self.get_keybings();

        if let Event::Key(CusKey::Esc) = event {
            self.handle_esc_key();
            return None;
        }
        if let Event::Key(CusKey::Enter) = event {
            self.handle_enter_key();
            return None;
        }

        if let Mode::Insert = self.cur_mode {
            if let Event::Key(key) = event {
                self.handle_user_input(key);
            }
            return None;
        }

        match event {
            Event::Tick => keybind.tick,
            Event::Error => keybind.tick,
            Event::Key(key) => match key {
                CusKey::Tab => keybind.tab,
                CusKey::E => {
                    self.cur_route = Route::PodList;
                    self.cur_mode = Mode::Insert;
                    self.user_input.clear();
                    None
                }
                CusKey::N => {
                    self.cur_route = Route::PodNamespace;
                    self.namespace_items.fixed = false;
                    self.user_input.clear();
                    None
                }
                CusKey::J => {
                    self.metrics_buffer.select_all();
                    self.metrics_buffer.cut();
                    keybind.j
                }
                CusKey::K => {
                    self.metrics_buffer.select_all();
                    self.metrics_buffer.cut();
                    keybind.k
                }
                CusKey::L => {
                    self.executor = Some(Executor::new(keybind.l, false, self.cur_route));
                    self.cur_route = Route::PodLog;
                    None
                } // show log
                CusKey::F => {
                    self.cur_route = Route::PodList;
                    self.cur_mode = Mode::Insert;
                    None
                } // 检索podlist 赛选
                CusKey::T => {
                    // 进入terminal 模式
                    self.executor = Some(Executor::new(keybind.t, false, self.cur_route));
                    self.cur_route = Route::PodTerm;
                    self.cur_mode = Mode::Insert;
                    None
                }
                CusKey::Q => keybind.q,
                CusKey::Enter => {
                    if let Route::PodNamespace = self.cur_route {
                        self.namespace_items.fixed = true;
                        self.cur_route = Route::PodIndex;
                    }
                    None
                }
                _ => keybind.tick,
            },
        }
    }

    // 根据当前的route id来获取对应的keybinds
    fn get_keybings(&self) -> KeyBinding {
        if self.cur_route >= Route::PodIndex && self.cur_route <= Route::PodEnd {
            return DEFAULT_POD_KEYBINDING;
        }
        if self.cur_route >= Route::DeployIndex && self.cur_route <= Route::DeployEnd {
            return DEFAULT_DEPLOYMENT_KEYBINDING;
        }
        if self.cur_route >= Route::NodeIndex && self.cur_route <= Route::NodeEnd {
            return DEFAULT_NODE_KEYBINDING;
        }
        DEFAULT_NOOP_KEYBINDING
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
            if self.cur_route == Route::PodLog {
                self.cur_route = Route::PodIndex;
            }
            executor.stop_fn.cancel();
        }
        match self.cur_route {
            d if d >= Route::PodIndex && d <= Route::PodEnd && d != Route::PodLog => {
                self.user_input.clear()
            }
            _ => {}
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
                self.user_input.done();
            }
            Mode::Command => {}
        }
        // insert模式下 ,只需要处理inputchar
        // command模式下,只需要处理command
    }

    fn handle_user_input(&mut self, key: CusKey) {
        match key {
            CusKey::Backspace => {
                if !self.user_input.is_empty() {
                    self.user_input.pop();
                }
            }
            _ => self.user_input.push(key.char()),
        }
    }

    pub fn handle_pod_metrics(&mut self, pod_metrics: PodMetrics) {
        let namespace = pod_metrics.meta().namespace.as_deref().unwrap();
        let pod_name = pod_metrics.meta().name.as_deref().unwrap();
        let namespace_cache = self
            .pod_metrics_cache
            .entry(namespace.to_string())
            .or_default();
        let metrics_cache = namespace_cache
            .entry(pod_name.to_string())
            .or_insert(CycledCache::<(i64, f64, f64)>::with_capacity(60));

        // todo: only get the first container metrics as pod metrics
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
            self.metrics_buffer.insert_str(metric_line);
            self.metrics_buffer.insert_newline();
        }
    }

    fn resync_cache_items(&mut self) {
        if self.user_input.fixed && !self.user_input.is_empty() {
            return;
        }
        let namespace = self
            .namespace_items
            .items
            .get(self.namespace_items.index)
            .unwrap();
        let items = self.store_pods.all_keys(namespace);
        if self.cur_route as i32 == Route::PodList as i32 && !self.user_input.is_empty() {
            let filter_items = Atom::new(
                self.user_input.as_str(),
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
        self.cache_items
            .items
            .get(self.cache_items.index)
            .map(|pod| (namespace.as_ref().unwrap() as &str, pod.as_ref()))
    }

    pub fn show_handle_pod_metrics(&self) -> bool {
        self.cur_route >= Route::PodIndex && self.cur_route <= Route::PodEnd
    }

    pub fn cancel_executor(&mut self) {
        if let Some(executor) = self.executor.take() {
            executor.stop_fn.cancel();
        }
    }

    pub fn pod_name(&self) -> Option<(Rc<str>, Rc<str>)> {
        if self.cur_route > Route::PodEnd {
            return None;
        }
        if let Some(namespace) = self.namespace_items.current_item() {
            if let Some(pod_name) = self.cache_items.current_item() {
                return Some((namespace, pod_name));
            }
        }
        None
    }
}

// cycled buffer , avoid memory allocate
pub struct CycledCache<T> {
    pub items: Vec<T>,
    capacity: usize,
    start_index: usize,
}

impl<T: Clone> CycledCache<T> {
    fn with_capacity(capacity: usize) -> Self {
        CycledCache {
            items: Vec::<T>::new(),
            capacity,
            start_index: 0,
        }
    }
    fn append(&mut self, obj: T) {
        if self.items.len() == self.capacity {
            *self.items.get_mut(self.start_index).unwrap() = obj;
            self.start_index = (self.start_index + 1) % self.capacity;
        } else {
            self.items.push(obj);
        }
    }
    pub fn get_all(&self) -> Vec<T> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.items[self.start_index..]);
        result.extend_from_slice(&self.items[..self.start_index]);
        result
    }

    pub fn get_all_limit(&self, limits: usize) -> Vec<T> {
        if limits >= self.capacity {
            return self.get_all();
        }

        let mut result = Vec::with_capacity(limits);
        if self.start_index + limits < self.capacity {
            if self.start_index + limits < self.items.len() {
                result.extend_from_slice(&self.items[self.start_index..self.start_index + limits]);
            } else {
                result.extend_from_slice(&self.items[self.start_index..]);
            }
        } else {
            result.extend_from_slice(&self.items[self.start_index..]);
            result.extend_from_slice(&self.items[..self.start_index + limits - self.capacity])
        }
        result
    }
}

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

impl StatefulList {
    fn current_item(&self) -> Option<Rc<str>> {
        if let Some(item) = self.items.get(self.index) {
            Some(item.clone())
        } else {
            None
        }
    }
}

pub struct KubeDescribeIndices<T> {
    indices: HashMap<String, HashMap<String, T>>,
}

impl<T> KubeDescribeIndices<T> {
    fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }
    pub fn get(&self, namespace: &str, name: &str) -> Option<&T> {
        let store = self.indices.get(namespace)?;
        store.get(name)
    }

    fn add(&mut self, namespace: String, name: String, obj: T) {
        if self.indices.get(namespace.as_str()).is_none() {
            let cache = HashMap::from([(name.to_string(), obj)]);
            self.indices.insert(namespace, cache);
        } else {
            self.indices
                .get_mut(namespace.as_str())
                .unwrap()
                .insert(name, obj);
        }
    }
    fn remove(&mut self, namespace: String, name: String, _obj: T) {
        if let Some(store) = self.indices.get_mut(namespace.as_str()) {
            store.remove(&name);
        }
    }
}

pub(super) struct Executor {
    pub run_fn: Option<HandleFn>,
    pub stop_fn: CancellationToken,
    state: bool,
    pub prev_route: Route,
}

impl Executor {
    fn new(hander: Option<HandleFn>, _is_running: bool, prev_route: Route) -> Executor {
        Executor {
            run_fn: hander,
            state: false,
            stop_fn: CancellationToken::new(),
            prev_route,
        }
    }
}

#[derive(Default)]
pub struct UserInput {
    buffer: String,
    fixed: bool,
}

// UserInput[#TODO] (should add some comments)
impl UserInput {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.fixed = false;
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    fn pop(&mut self) {
        self.buffer.pop();
    }

    fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
    }
    fn done(&mut self) {
        self.fixed = true;
    }
    pub fn clone(&self) -> String {
        self.buffer.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_appendonly_cache() {
        let mut cache = CycledCache::<i32>::with_capacity(10);
        for i in 0..12 {
            cache.append(i);
        }
        assert_eq!(vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11], cache.get_all());

        assert_eq!(vec![2, 3], cache.get_all_limit(2));

        assert_eq!(
            vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            cache.get_all_limit(20)
        );
    }
}
