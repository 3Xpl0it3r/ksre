use std::fmt::Debug;

use tokio_util::sync::CancellationToken;

use crate::App;

/* pub type HandleFn = fn(&mut App) -> Pin<Box<dyn Future<Output = ()>>>; */
pub type HandleFn = fn(&mut App, cancel_fn: Option<CancellationToken>);
pub type Handler = Option<HandleFn>;

macro_rules! keybind_struct {
    ($($key:ident),*) => {
        pub struct KeyBinding {
            $(pub $key: Handler),*
        }
    };
}

keybind_struct!(
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, tab, tick
);
macro_rules! generate_keybinding {
    (@default_keybinding  $( $ele:ident ),* ) => {
        KeyBinding {
            $( $ele: None ),*
        }
    };
    (@empty) => {
        generate_keybinding!(@default_keybinding a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, tab, tick)
    };
    ($( $key:ident : $value:path ),*) => {
        {
            let mut empty_keybindings = generate_keybinding!(@empty);
            $(empty_keybindings.$key = Some($value));*;
            empty_keybindings
        }
    };
}

pub const DEFAULT_POD_KEYBINDING: KeyBinding = generate_keybinding! {
    j: App::select_items_next,
    k: App::select_items_prev,
    l: App::handle_pod_logs,
    q: App::handle_quit,
    t: App::handle_pod_exec

};

pub const DEFAULT_DEPLOYMENT_KEYBINDING: KeyBinding = generate_keybinding! {
    q: App::handle_quit
};

pub const DEFAULT_NODE_KEYBINDING: KeyBinding = generate_keybinding! {
    q: App::handle_quit
};

pub const DEFAULT_NOOP_KEYBINDING: KeyBinding = generate_keybinding!(@empty);
