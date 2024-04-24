use std::{char, collections::HashMap, rc::Rc, sync::Arc};

use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use kube::Client as KubeClient;
use nucleo_matcher::{Config, Matcher};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tui_textarea::TextArea;

use crate::kubernetes::{api::pod::PodDescribe, indexer::StoreIndex};

pub struct AppState {
    kube_client: KubeClient,
    cur_mode: Mode, //当前模式
    tabpage: TabPage,
    route: Route,

    pub fuzz_matcher: Matcher,
    pub user_input: UserInput,
    pub pod_storage: StoreIndex<PodSpec, PodStatus>,
    pub pod_describes: KubeDescribeIndices<PodDescribe>,
    pub cache_items: StatefulList,
    pub namespace_cache: StatefulList,
    pub nodes_cache: StatefulList,
    pub pod_metrics_cache: HashMap<String, HashMap<String, CycledCache<(i64, f64, f64)>>>,
    pub stdout_buffer: Arc<tokio::sync::RwLock<TextArea<'static>>>,
    pub metrics_buffer: TextArea<'static>,
    pub executor: Option<Executor>,

    // format is timestamp, cpu, memory
    quit: bool,
}

impl AppState {
    pub fn new(kube_client: KubeClient) -> Self {
        let mut metrics_buffer = TextArea::default();
        metrics_buffer.set_max_histories(32);
        Self {
            kube_client,
            cur_mode: Mode::Normal,
            route: Route::PodIndex,
            cache_items: StatefulList::default(),
            namespace_cache: StatefulList::default(),
            nodes_cache: StatefulList::default(),
            user_input: UserInput::default(),
            fuzz_matcher: Matcher::new(Config::DEFAULT),
            tabpage: TabPage::Pod,
            pod_storage: StoreIndex::new(),
            pod_describes: KubeDescribeIndices::new(),
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
    pub fn kube_client(&self) -> KubeClient {
        self.kube_client.clone()
    }
}

// AppState[#TODO] (should add some comments)
impl AppState {
    pub fn next_route(&mut self) {
        // switch route will stop all executors
        self.executor.take();
        // clean all relative buffer
        self.user_input.clear();
        self.cache_items.reset();
        // clean all metrics buffer
        self.metrics_buffer.select_all();
        self.metrics_buffer.cut();

        self.tabpage = self.tabpage.next();
    }

    #[inline]
    pub fn get_tabpage(&self) -> TabPage {
        self.tabpage
    }

    #[inline]
    pub fn get_route(&self) -> Route {
        self.route
    }

    #[inline]
    pub fn set_route(&mut self, route: Route) {
        self.route = route
    }
    #[inline]
    pub fn get_mode(&self) -> Mode {
        self.cur_mode
    }
    #[inline]
    pub fn set_mode(&mut self, mode: Mode) {
        self.cur_mode = mode
    }

    pub fn stop_executor(&mut self) {
        self.executor.take();
    }
}

// handlers
impl AppState {
    pub fn handle_quit(&mut self) {
        self.quit = true;
    }
}

impl AppState {
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}

pub struct Executor {
    pub normal_task: Option<Box<dyn FnMut()>>,
    pub stop_fn: Option<CancellationToken>,
    pub async_task: Option<Vec<JoinHandle<()>>>,
    pub _type: bool, // true -> commond task, false mean async_task
}

impl Executor {
    pub fn execute(&mut self) {
        match self._type {
            true => {
                self.normal_task.as_mut().unwrap()();
            }
            false => {}
        }
    }
}

// Drop[#TODO] (should add some comments)
impl Drop for Executor {
    fn drop(&mut self) {
        if let Some(cancellation_token) = self.stop_fn.take() {
            if !cancellation_token.is_cancelled() {
                cancellation_token.cancel();
            }
        }
        if let Some(async_task) = self.async_task.take() {
            for single_task in async_task.into_iter() {
                if single_task.is_finished() {
                    single_task.abort();
                }
            }
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

#[derive(Default)]
pub struct StatefulList {
    items: Vec<Rc<str>>,
    index: usize,
    confirmed: bool,
}

impl StatefulList {
    #[inline]
    pub fn push(&mut self, item: Rc<str>) {
        self.items.push(item);
    }
    #[inline]
    pub fn get(&self) -> Option<Rc<str>> {
        self.items.get(self.index).cloned()
    }
    pub fn replace(&mut self, items: Vec<Rc<str>>) {
        self.items = items;
    }
    #[inline]
    pub fn list(&self) -> &Vec<Rc<str>> {
        &self.items
    }
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn reindex(&mut self) {
        self.index = 0;
    }
    #[inline]
    pub fn reset(&mut self) {
        self.confirmed = false;
        self.items.clear();
        self.index = 0;
    }
    #[inline]
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }
    #[inline]
    pub fn un_confirm(&mut self) {
        self.confirmed = false;
    }
    #[inline]
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }
    #[inline]
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
    #[inline]
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

    pub fn add(&mut self, namespace: String, name: String, obj: T) {
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
}

pub struct UserInput {
    buffer: String,
    completed: bool, // true represent input op has done
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            completed: true,
        }
    }
}

impl UserInput {
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.completed = true;
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    #[inline]
    pub fn pop(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
        }
        self.completed = false;
    }

    #[inline]
    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
        self.completed = false;
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
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

#[derive(Clone, Copy, Debug)]
pub enum TabPage {
    Pod,
    Deploy,
    Node,
}

impl TabPage {
    pub fn next(self) -> Self {
        match self {
            TabPage::Pod => TabPage::Deploy,
            TabPage::Deploy => TabPage::Node,
            TabPage::Node => TabPage::Pod,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            TabPage::Pod => TabPage::Node,
            TabPage::Deploy => TabPage::Pod,
            TabPage::Node => TabPage::Pod,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Route {
    PodIndex,
    PodNamespace,
    PodList,
    PodState,
    PodLog,
    PodTerm,

    DeployIndex,
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
