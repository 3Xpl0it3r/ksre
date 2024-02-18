use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;
use tui_textarea::TextArea;

use crate::app::{
    action::Route,
    state::AppState,
    ui::{
        pod::{draw_page_pod_list, draw_page_pod_status, draw_pod_logs, draw_pod_resource},
        util as uiutil,
    },
};
use crate::kubernetes::api::PodDescribe;

// -------------------------------------
// ---- pods          | resource       |
// [namespace] [node] | resource       |
// [pod list]         | cpu memory net |
// ------------------------------------|
// devops
// logs| exec

pub fn draw_page_index(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let chunks = uiutil::vertical_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        area,
    );

    // pod_chunks[0] for pod list, pod_chunks【1】 is used to render pod status
    let main_area = uiutil::horizontal_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        chunks[0],
    );

    let pod_list_area = uiutil::outer_block(f, "Pods", main_area[0]);
    let pod_res_area = uiutil::outer_block(f, "Pod Resource", main_area[1]);
    let devops_chunk = uiutil::outer_block(
        f,
        "Debug [l] show log, [t] attach terminal  [esc] back to home",
        chunks[1],
    );
    // pod_chunks[0] 展示pod list, ,pod_chunks[1] 展示 pod status
    //
    draw_page_pod_list(f, pod_list_area, state);

    let namespace = state
        .namespace_items
        .items
        .get(state.namespace_items.index)
        .unwrap();

    draw_pod_resource(f, state, None, pod_res_area);

    if let Some(pod) = state.cache_items.items.get(state.cache_items.index) {
        let obj = state.store_pods.get_value(namespace, pod);

        if obj.is_some() {
            let pod_describe = Some(PodDescribe::from(obj.as_ref().unwrap()));

            match state.cur_route {
                Route::PodLog => draw_pod_logs(f, state, pod_describe.as_ref(), devops_chunk, reader),
                Route::PodTerm => {}
                _ => draw_page_pod_status(f, state, pod_describe.as_ref(), devops_chunk),
            }
            return;
        }
    }
    match state.cur_route {
        Route::PodLog => draw_pod_logs(f, state, None, devops_chunk, reader),
        Route::PodTerm => {}
        _ => draw_page_pod_status(f, state, None, devops_chunk),
    }
}
