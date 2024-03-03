use crate::app::state::{AppState, Executor, Mode, Route};

pub fn handle_quit(app_state: &mut AppState) -> Option<Executor> {
    app_state.handle_quit()
}

pub fn handle_esc_key(app_state: &mut AppState) -> Option<Executor> {
    match app_state.get_route() {
        Route::PodLog => {
            app_state.stop_executor();
            app_state.update_route(Route::PodIndex);
        }
        _ => app_state.update_route(Route::PodIndex),
    }
    None
}

pub fn handle_enter_key(app_state: &mut AppState) -> Option<Executor> {
    if let Route::PodNamespace = app_state.get_route() {
        app_state.update_route(Route::PodList);
    }
    None
}

pub fn trigger_userinput(app_state: &mut AppState) -> Option<Executor> {
    match app_state.get_route() {
        Route::PodIndex | Route::PodList | Route::PodState => {
            app_state.update_mode(Mode::Insert);
        }
        _ => {}
    }
    None
}

pub fn trigger_namespace_select(app_state: &mut AppState) -> Option<Executor> {
    app_state.update_route(Route::PodNamespace);
    None
}

pub fn select_next_item(app_state: &mut AppState) -> Option<Executor> {
    match app_state.get_route() {
        Route::PodNamespace => {
            app_state.namespace_items.next();
        }
        Route::PodIndex | Route::PodList => {
            app_state.cache_items.next();
        }
        _ => {}
    }
    None
}

pub fn select_prev_item(app_state: &mut AppState) -> Option<Executor> {
    match app_state.get_route() {
        Route::PodNamespace => {
            app_state.namespace_items.prev();
        }
        Route::PodIndex | Route::PodList => {
            app_state.cache_items.prev();
        }
        _ => {}
    }
    None
}

pub fn show_pod_log(app_state: &mut AppState) -> Option<Executor> {
    None
}
