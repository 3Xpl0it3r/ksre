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
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tui_textarea::TextArea;

use crate::{
    event::{CusKey, Event, KubeEvent},
    kubernetes::api::metrics::PodMetrics,
    kubernetes::api::PodDescribe,
    kubernetes::{api::RtObject, indexer::StoreIndex},
};

use super::handler::POD_MAPPINGS;

pub struct AppState {
    cur_mode: Mode, //当前模式
    pub user_input: UserInput,
    pub fuzz_matcher: Matcher,
    // 当前选中的tab页面
    cur_route: Route,
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

    quit: bool,
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
            cur_route: Route::PodIndex,
            cur_pod: 0,
            cur_node: 0,
            store_pods: StoreIndex::new(),
            kube_obj_describe_cache: KubeDescribeIndices::new(),
            stdout_buffer: Arc::new(tokio::sync::RwLock::new(TextArea::default())),
            metrics_buffer,
            executor: None,
            pod_metrics_cache: HashMap::new(),
            quit: false,
        }
    }
}

// kubernetes 事件处理
impl AppState {
    // 同步来自接收apiserver 数据事件
    pub fn handle_podevents(&mut self, event: KubeEvent<PodSpec, PodStatus>) -> Option<Executor> {
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

    #[inline]
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
    #[inline]
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
    pub fn dispatch_keyevents(&mut self, event: Event) -> Option<Executor> {
        match event {
            Event::Key(key_char) => {
                // 优先处理用户输入
                if Mode::Insert as i32 == self.cur_mode as i32 && !self.handle_user_input(key_char)
                {
                    return None;
                }
                // 第二优先级处理tab键
                if let CusKey::Tab = key_char {
                    self.next_route();
                    return None;
                }
                // 第三开始dispatch到具体窗口handler来处理对应的keyevent
                if self.cur_route.to_pod() {
                    self.sync_cacheitems_for_pods();
                    if let Some(f) = POD_MAPPINGS.get(key_char.as_ref()) {
                        f(self)
                    } else {
                        None
                    }
                } else if self.cur_route.to_deployment() {
                    self.sync_cacheitems_for_deployments();
                    None
                } else if self.cur_route.to_node() {
                    self.sync_cacheitems_for_nodes();
                    None
                } else {
                    None
                }
            }
            Event::Tick => {
                if self.cur_route.to_pod() {
                    self.sync_cacheitems_for_pods();
                    None
                } else if self.cur_route.to_deployment() {
                    self.sync_cacheitems_for_deployments();
                    None
                } else if self.cur_route.to_node() {
                    self.sync_cacheitems_for_nodes();
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn add_metrics(&mut self, pod_metrics: PodMetrics) {
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

    #[inline]
    fn sync_cacheitems_for_pods(&mut self) {
        let namespace = self.get_namespace().unwrap();
        let items = self.store_pods.all_keys(namespace.as_ref());
        if !self.user_input.is_empty() {
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
        } else {
            self.cache_items.items = items;
        }
        if !self.user_input.is_completed() {
            self.cache_items.index = 0;
        }
    }

    #[inline]
    fn sync_cacheitems_for_deployments(&mut self) {}
    #[inline]
    fn sync_cacheitems_for_nodes(&mut self) {}

    #[inline]
    fn handle_user_input(&mut self, key: CusKey) -> bool {
        // true ,input has done
        match key {
            CusKey::Backspace => {
                self.user_input.pop();
                false
            }
            CusKey::Enter | CusKey::Esc => {
                self.user_input.complete();
                self.cur_mode = Mode::Normal;
                true
            }
            _ => {
                self.user_input.push(key.char());
                false
            }
        }
    }

    pub fn next_route(&mut self) {
        // switch route will stop all executors
        if let Some(executor) = self.executor.take() {
            executor.stop()
        }
        // clean all relative buffer
        self.user_input.clear();
        self.cache_items.reset();
        // clean all metrics buffer
        self.metrics_buffer.select_all();
        self.metrics_buffer.cut();

        self.cur_route = self.cur_route.next();
    }

    #[inline]
    pub fn get_route(&self) -> Route {
        self.cur_route
    }
    #[inline]
    pub fn update_route(&mut self, route: Route) {
        self.cur_route = route
    }
    #[inline]
    pub fn get_mode(&self) -> Mode {
        self.cur_mode
    }
    #[inline]
    pub fn update_mode(&mut self, mode: Mode) {
        self.cur_mode = mode
    }

    pub fn get_namespace(&self) -> Option<Rc<str>> {
        self.namespace_items
            .items
            .get(self.namespace_items.index)
            .cloned()
    }

    pub fn stop_executor(&mut self) {
        if let Some(executor) = self.executor.take() {
            executor.stop_fn.cancel();
        }
    }

    pub fn get_pod(&self) -> Option<Rc<str>> {
        if self.cur_route > Route::PodEnd {
            return None;
        }
        self.cache_items.current_item()
    }

    pub fn initial_namespaces(&mut self, ns_name: Rc<str>) {
        self.namespace_items.items.push(ns_name);
    }

    pub fn list_namespace(&self) -> &StatefulList {
        &self.namespace_items
    }

    pub fn clean_user_input(&mut self) {
        self.user_input.clear();
    }



    pub fn namespace_sts_confirm(&mut self) {
        self.namespace_items.confirm();
    }
    pub fn namespace_sts_is_confirmed(&self)-> bool {
        self.namespace_items.is_confirmed()
    }

}

// handlers
impl AppState {
    pub fn handle_quit(&mut self) -> Option<Executor> {
        self.quit = true;
        None
    }
}

impl AppState {
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}

pub struct Executor {
    normal_task: Option<fn(&AppState)>,
    stop_fn: CancellationToken,
    async_task: JoinHandle<()>,
    e_type: bool, // true -> commond task, false mean async_task
}

impl Executor {
    pub fn execute(&self) {}
    fn stop(self) {
        if self.stop_fn.is_cancelled() {
            self.stop_fn.cancel()
        }
        if self.async_task.is_finished() {
            self.async_task.abort()
        }
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
    pub confirmed: bool,
}

impl StatefulList {
    fn current_item(&self) -> Option<Rc<str>> {
        self.items.get(self.index).cloned()
    }

    fn reset(&mut self) {
        self.confirmed = false;
        self.items.clear();
        self.index = 0;
    }
    fn is_confirmed(&self) -> bool {
        self.confirmed
    }
    fn confirm(&mut self) {
        self.confirmed = true;
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

pub struct UserInput {
    buffer: String,
    completed: bool, // true represent input op has done
}

// Default[#TODO] (should add some comments)
impl Default for UserInput {
    fn default() -> Self {
        Self { buffer: String::new(), completed: true }
    }
}

impl UserInput {
    #[inline]
    fn clear(&mut self) {
        self.buffer.clear();
        self.completed = true;
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    #[inline]
    fn pop(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
        }
        self.completed = false;
    }

    #[inline]
    fn push(&mut self, c: char) {
        self.buffer.push(c);
        self.completed = false;
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
    }
    #[inline]
    pub fn clone(&self) -> String {
        self.buffer.clone()
    }
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.completed
    }
    #[inline]
    pub fn complete(&mut self) {
        self.completed = true;
    }
}

const ROUTE_STEP: isize = 100;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    PodIndex = 0, // pod begin
    PodNamespace,
    PodList,
    PodState,
    PodTerm,
    PodLog,
    PodEnd, // pod end
    DeployIndex = ROUTE_STEP,
    DeployEnd,
    NodeIndex = 2 * ROUTE_STEP,
    NodeEnd,
    PlaceHolder = 3 * ROUTE_STEP,
}

impl Route {
    #[inline]
    pub fn route_step() -> isize {
        ROUTE_STEP
    }
    #[inline]
    pub fn to_pod(self) -> bool {
        self >= Route::PodIndex && self <= Route::PodEnd
    }
    #[inline]
    pub fn to_deployment(self) -> bool {
        self >= Route::DeployIndex && self <= Route::DeployEnd
    }
    #[inline]
    pub fn to_node(self) -> bool {
        self >= Route::NodeIndex && self <= Route::NodeEnd
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this = *self as isize;
        let another = *other as isize;
        Some(this.cmp(&another))
    }
}

impl Route {
    pub fn next(self) -> Self {
        if self as usize == 200 {
            Route::PodIndex
        } else {
            let c_tb_nr = self as usize;
            match c_tb_nr {
                0 => Route::DeployIndex,
                100 => Route::NodeIndex,
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Insert,
    Normal,
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
