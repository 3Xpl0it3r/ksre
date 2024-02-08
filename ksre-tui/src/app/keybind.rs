use std::fmt::Debug;

use tokio_util::sync::CancellationToken;

use crate::App;

use crate::app::action::Route;

/* pub type HandleFn = fn(&mut App) -> Pin<Box<dyn Future<Output = ()>>>; */
pub type HandleFn = fn(&mut App, cancel_fn: Option<CancellationToken>);

#[derive(Debug)]
pub struct KeyContext {
    pub route_id: Route,
    pub guard: Route,
    pub handler: Option<HandleFn>,
}

pub const KEY_CONTEXT_RECONCILE: KeyContext = KeyContext {
    route_id: Route::Any,
    guard: Route::Unreach,
    handler: None,
};

impl Default for KeyContext {
    fn default() -> Self {
        KeyContext {
            route_id: Route::PodIndex,
            guard: Route::PodEnd,
            handler: None,
        }
    }
}

#[derive(Default)]
pub struct KeyBindings {
    pub a: KeyContext,
    pub b: KeyContext,
    pub c: KeyContext,
    pub d: KeyContext,
    pub e: KeyContext,
    pub f: KeyContext,
    pub g: KeyContext,
    pub h: KeyContext,
    pub j: KeyContext,
    pub k: KeyContext,
    pub l: KeyContext,
    pub m: KeyContext,
    pub n: KeyContext,
    pub o: KeyContext,
    pub p: KeyContext,
    pub q: KeyContext,
    pub s: KeyContext,
    pub t: KeyContext,
    pub u: KeyContext,
    pub v: KeyContext,
    pub w: KeyContext,
    pub x: KeyContext,
    pub y: KeyContext,
    pub z: KeyContext,
    pub tab: KeyContext,
    pub tick: KeyContext,
}

pub const DEFAULT_ERROR_HANDLE: KeyContext = KeyContext {
    route_id: Route::Unreach,
    guard: Route::Error,
    handler: None,
};

// pod页面相关的keybinds配置
pub const DEFAULT_POD_KEYBIND: KeyBindings = KeyBindings {
    a: KeyContext {
        route_id: Route::PodIndex,
        guard: Route::PodEnd,
        handler: Some(App::fake_handlefunction),
    },
    tab: KeyContext {
        route_id: Route::PodIndex,
        guard: Route::PodEnd,
        handler: Some(App::handle_next_route),
    },
    b: DEFAULT_ERROR_HANDLE,
    c: DEFAULT_ERROR_HANDLE,
    d: DEFAULT_ERROR_HANDLE,
    e: DEFAULT_ERROR_HANDLE,
    f: KeyContext { route_id: Route::PodIndex, guard: Route::PodEnd, handler: None },
    g: DEFAULT_ERROR_HANDLE,
    h: DEFAULT_ERROR_HANDLE,
    j: KeyContext {
        route_id: Route::PodIndex,
        guard: Route::PodIndex,
        handler: Some(App::pod_list_next_item),
    },
    k: KeyContext {
        route_id: Route::PodIndex,
        guard: Route::PodIndex,
        handler: Some(App::pod_list_prev_item),
    },
    l: DEFAULT_ERROR_HANDLE,
    m: DEFAULT_ERROR_HANDLE,
    n: DEFAULT_ERROR_HANDLE,
    o: DEFAULT_ERROR_HANDLE,
    p: DEFAULT_ERROR_HANDLE,
    q: KeyContext {
        route_id: Route::Any,
        handler: Some(App::handle_quit),
        guard: Route::PodEnd,
    },
    s: DEFAULT_ERROR_HANDLE,
    t: KeyContext {
        route_id: Route::PodIndex,
        guard: Route::PodEnd,
        handler: Some(App::handle_pod_exec),
    },
    u: DEFAULT_ERROR_HANDLE,
    v: DEFAULT_ERROR_HANDLE,
    w: DEFAULT_ERROR_HANDLE,
    x: DEFAULT_ERROR_HANDLE,
    y: DEFAULT_ERROR_HANDLE,
    z: DEFAULT_ERROR_HANDLE,
    tick: DEFAULT_ERROR_HANDLE,
};

// deployment界面的keybind配置
pub const DEFAULT_DPL_KEYBIND: KeyBindings = KeyBindings {
    a: KeyContext {
        route_id: Route::DeployIndex,
        guard: Route::DeployEnd,
        handler: Some(App::fake_handlefunction),
    },
    tab: KeyContext {
        route_id: Route::DeployIndex,
        guard: Route::DeployEnd,
        handler: Some(App::handle_next_route),
    },
    b: DEFAULT_ERROR_HANDLE,
    c: DEFAULT_ERROR_HANDLE,
    d: DEFAULT_ERROR_HANDLE,
    e: DEFAULT_ERROR_HANDLE,
    f: DEFAULT_ERROR_HANDLE,
    g: DEFAULT_ERROR_HANDLE,
    h: DEFAULT_ERROR_HANDLE,
    j: DEFAULT_ERROR_HANDLE,
    k: DEFAULT_ERROR_HANDLE,
    l: DEFAULT_ERROR_HANDLE,
    m: DEFAULT_ERROR_HANDLE,
    n: DEFAULT_ERROR_HANDLE,
    o: DEFAULT_ERROR_HANDLE,
    p: DEFAULT_ERROR_HANDLE,
    q: KeyContext {
        route_id: Route::DeployIndex,
        guard: Route::DeployEnd,
        handler: Some(App::handle_quit),
    },
    s: DEFAULT_ERROR_HANDLE,
    t: DEFAULT_ERROR_HANDLE,
    u: DEFAULT_ERROR_HANDLE,
    v: DEFAULT_ERROR_HANDLE,
    w: DEFAULT_ERROR_HANDLE,
    x: DEFAULT_ERROR_HANDLE,
    y: DEFAULT_ERROR_HANDLE,
    z: DEFAULT_ERROR_HANDLE,
    tick: DEFAULT_ERROR_HANDLE,
};
// 节点相关的配置
pub const DEFAULT_NODE_KEYBIND: KeyBindings = KeyBindings {
    a: KeyContext {
        route_id: Route::NodeIndex,
        guard: Route::NodeEnd,
        handler: Some(App::fake_handlefunction),
    },
    tab: KeyContext {
        route_id: Route::NodeIndex,
        guard: Route::NodeEnd,
        handler: Some(App::handle_next_route),
    },
    b: DEFAULT_ERROR_HANDLE,
    c: DEFAULT_ERROR_HANDLE,
    d: DEFAULT_ERROR_HANDLE,
    e: DEFAULT_ERROR_HANDLE,
    f: DEFAULT_ERROR_HANDLE,
    g: DEFAULT_ERROR_HANDLE,
    h: DEFAULT_ERROR_HANDLE,
    j: DEFAULT_ERROR_HANDLE,
    k: DEFAULT_ERROR_HANDLE,
    l: DEFAULT_ERROR_HANDLE,
    m: DEFAULT_ERROR_HANDLE,
    n: DEFAULT_ERROR_HANDLE,
    o: DEFAULT_ERROR_HANDLE,
    p: DEFAULT_ERROR_HANDLE,
    q: KeyContext {
        guard: Route::NodeEnd,
        route_id: Route::NodeIndex,
        handler: Some(App::handle_quit),
    },
    s: DEFAULT_ERROR_HANDLE,
    t: DEFAULT_ERROR_HANDLE,
    u: DEFAULT_ERROR_HANDLE,
    v: DEFAULT_ERROR_HANDLE,
    w: DEFAULT_ERROR_HANDLE,
    x: DEFAULT_ERROR_HANDLE,
    y: DEFAULT_ERROR_HANDLE,
    z: DEFAULT_ERROR_HANDLE,
    tick: DEFAULT_ERROR_HANDLE,
};
// 预留不存在的key
pub const DEFAULT_NOP_KEYBINDS: KeyBindings = KeyBindings {
    a: DEFAULT_ERROR_HANDLE,
    tab: DEFAULT_ERROR_HANDLE,
    b: DEFAULT_ERROR_HANDLE,
    c: DEFAULT_ERROR_HANDLE,
    d: DEFAULT_ERROR_HANDLE,
    e: DEFAULT_ERROR_HANDLE,
    f: DEFAULT_ERROR_HANDLE,
    g: DEFAULT_ERROR_HANDLE,
    h: DEFAULT_ERROR_HANDLE,
    j: DEFAULT_ERROR_HANDLE,
    k: DEFAULT_ERROR_HANDLE,
    l: DEFAULT_ERROR_HANDLE,
    m: DEFAULT_ERROR_HANDLE,
    n: DEFAULT_ERROR_HANDLE,
    o: DEFAULT_ERROR_HANDLE,
    p: DEFAULT_ERROR_HANDLE,
    q: DEFAULT_ERROR_HANDLE,
    s: DEFAULT_ERROR_HANDLE,
    t: DEFAULT_ERROR_HANDLE,
    u: DEFAULT_ERROR_HANDLE,
    v: DEFAULT_ERROR_HANDLE,
    w: DEFAULT_ERROR_HANDLE,
    x: DEFAULT_ERROR_HANDLE,
    y: DEFAULT_ERROR_HANDLE,
    z: DEFAULT_ERROR_HANDLE,
    tick: DEFAULT_ERROR_HANDLE,
};
