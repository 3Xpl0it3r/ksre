use std::usize;

use super::AppState;

const ROUTE_STEP: isize = 100;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    PodIndex = 0,
    // -----pod begin
    PodNamespace,
    PodList,
    PodState,
    PodTerm,
    PodLog,
    // -----pod end
    PodEnd,
    DeployIndex = ROUTE_STEP,
    DeployEnd,
    NodeIndex = 2 * ROUTE_STEP,
    NodeEnd,
    Any = 3 * ROUTE_STEP,
    Flush,
    Unreach = 4 * ROUTE_STEP,
    Error,
}

impl Route {
    pub fn in_guard(self, cur_route: Route) -> bool {
        let guard = self as i32;
        let cur_id = cur_route as i32;
        return cur_id >= guard - ROUTE_STEP as i32 && cur_id <= guard;
    }
    pub fn route_step() -> isize {
        ROUTE_STEP
    }
}

// Ord[#TODO] (should add some comments)
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

pub struct RouteHandler {}
impl RouteHandler {
    pub fn pod_logs(state: &mut AppState, reader: Option<tokio::sync::RwLockReadGuard<String>>) {}

    pub fn pod_exec(state: &mut AppState, reader: Option<tokio::sync::RwLockReadGuard<String>>) {}

    pub fn route_switch(
        state: &mut AppState,
        reader: Option<tokio::sync::RwLockReadGuard<String>>,
    ) {
    }

    pub fn refresh(state: &mut AppState, reader: Option<tokio::sync::RwLockReadGuard<String>>) {}
}




// endit mode
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Insert,
    Normal,
    // in command module will disable all key event from tui terminal
    // when live command mode, appstate will empty all event buffer from tui event channel
    Command,
}
// Eq[#TODO] (should add some comments)
