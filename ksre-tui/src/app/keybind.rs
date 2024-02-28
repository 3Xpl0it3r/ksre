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
    (( $( $ele:ident ),* )) => {
        KeyBinding {
            $( $ele: None ),*
        }
    };
    (( $( $key:ident : $value:ident ),* )) => {
        KeyBinding {
            $( $key: Some($value) ),*
        }
    };
    (( $( $ele:ident ),+ ) , ( $($key:ident : $value:path),+ )) => {
        KeyBinding {
            $( $ele: None ),* ,
            $( $key: Some($value) ),*
        }
    };
}

pub const DEFAULT_POD_KEYBINDING: KeyBinding = generate_keybinding! {
    (a,b,c,d,e,f,g,h,i,m,n,o,p,r,s,u,v,w,x,y,z,tab, tick),
    // custom define keyactions
    (j : App::select_items_next, k: App::select_items_prev, l:App::handle_pod_logs, q:App::handle_quit, t: App::handle_pod_exec)

};

pub const DEFAULT_DEPLOYMENT_KEYBINDING: KeyBinding = generate_keybinding! {
    (a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,r,s,t,u,v,w,x,y,z,tab, tick),
    // custom define keyactions
    (q: App::handle_quit)
};

pub const DEFAULT_NODE_KEYBINDING: KeyBinding = generate_keybinding! {
    (a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,r,s,t,u,v,w,x,y,z,tab, tick),
    (q: App::handle_quit)
};

pub const DEFAULT_NOOP_KEYBINDING: KeyBinding = generate_keybinding! {
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, tab, tick)
};
