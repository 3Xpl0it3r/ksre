use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::app::{
    job::pod_log,
    state::{AppState, Executor, Mode, Route},
};

pub fn handle_quit(app_state: &mut AppState) -> Option<&mut Executor> {
    app_state.handle_quit();
    None
}

pub fn handle_esc_key(app_state: &mut AppState) -> Option<&mut Executor> {
    match app_state.get_route() {
        Route::PodLog => {
            app_state.stop_executor();
            app_state.set_route(Route::Pod);
        }
        _ => app_state.set_route(Route::Pod),
    }
    None
}

pub fn handle_enter_key(app_state: &mut AppState) -> Option<&mut Executor> {
    if let Route::PodNamespace = app_state.get_route() {
        app_state.namespace_cache.confirm();
        app_state.set_route(Route::PodList);
    }
    None
}

pub fn trigger_userinput(app_state: &mut AppState) -> Option<&mut Executor> {
    match app_state.get_route() {
        Route::Pod | Route::PodList | Route::PodState => {
            app_state.user_input.clear();
            app_state.set_mode(Mode::Insert);
        }
        _ => {}
    }
    None
}

pub fn trigger_namespace_select(app_state: &mut AppState) -> Option<&mut Executor> {
    app_state.set_route(Route::PodNamespace);
    None
}

pub fn select_next_item(app_state: &mut AppState) -> Option<&mut Executor> {
    match app_state.get_route() {
        Route::PodNamespace => {
            app_state.namespace_cache.next();
        }
        Route::Pod | Route::PodList => {
            app_state.cache_items.next();
        }
        _ => {}
    }
    None
}

pub fn select_prev_item(app_state: &mut AppState) -> Option<&mut Executor> {
    match app_state.get_route() {
        Route::PodNamespace => {
            app_state.namespace_cache.prev();
        }
        Route::Pod | Route::PodList => {
            app_state.cache_items.prev();
        }
        _ => {}
    }
    None
}

pub fn show_pod_log(app_state: &mut AppState) -> Option<&mut Executor> {
    if let Some(pod_name) = app_state.cache_items.get() {
        app_state.set_route(Route::PodLog);
        let cancellation_token = CancellationToken::default();
        let kube_client = app_state.kube_client();
        let namespace = app_state.namespace_cache.get().unwrap();
        let (log_writer_tx, mut log_reader_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) =
            mpsc::channel(10);
        let task0 = tokio::spawn(pod_log::tail_logs(
            cancellation_token.clone(),
            kube_client,
            log_writer_tx,
            pod_name.to_string(),
            namespace.to_string(),
        ));
        let writer = app_state.stdout_buffer.clone();
        let task1 = tokio::spawn(async move {
            {
                writer.write().await.select_all();
                writer.write().await.cut();
            }
            while let Some(line) = log_reader_rx.recv().await {
                writer.write().await.insert_str(line.as_str());
                writer.write().await.insert_newline();
            }
        });
        let executor = Executor {
            normal_task: None,
            stop_fn: Some(cancellation_token),
            async_task: Some(vec![task0, task1]),
            _type: false,
        };
        app_state.executor = Some(executor);
        app_state.executor.as_mut()
    } else {
        None
    }
}
