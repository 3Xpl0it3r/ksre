use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::app::state::{AppState, Executor};

/* use super::state::{AppState, Executor}; */
/* pub type HandleFn = fn(&mut App) -> Pin<Box<dyn Future<Output = ()>>>; */
/* pub type HandleFn = fn(&mut AppState); */
pub type Handler = fn(&mut AppState) -> Option<&mut Executor>;

macro_rules! key_binding {
    ($($key: literal : $value:path),*) => {
        {
            let mut keymaps = HashMap::new();
            $(keymaps.insert($key, $value as Handler));* ;
            keymaps
        }
    };
}

use super::pod;
lazy_static! {
    pub static ref POD_KEYMAPS: HashMap<&'static str, Handler> = key_binding! {
        "e": pod::trigger_userinput,
        "q": pod::handle_quit,
        "n": pod::trigger_namespace_select,
        "j": pod::select_next_item,
        "k": pod::select_prev_item,
        "l": pod::show_pod_log,
        "Esc": pod::handle_esc_key,
        "Enter": pod::handle_enter_key
    };
    pub static ref DEPLOYMENT_KEYMAPS: HashMap<&'static str, Handler> = key_binding! {
        "e": pod::trigger_userinput,
        "q": pod::handle_quit,
        "n": pod::trigger_namespace_select,
        "j": pod::select_next_item,
        "k": pod::select_prev_item,
        "l": pod::show_pod_log,
        "Esc": pod::handle_esc_key,
        "Enter": pod::handle_enter_key
    };
    pub static ref NODE_KEYMAPS: HashMap<&'static str, Handler> = key_binding! {
        "e": pod::trigger_userinput,
        "q": pod::handle_quit,
        "n": pod::trigger_namespace_select,
        "j": pod::select_next_item,
        "k": pod::select_prev_item,
        "l": pod::show_pod_log,
        "Esc": pod::handle_esc_key,
        "Enter": pod::handle_enter_key
    };
}
