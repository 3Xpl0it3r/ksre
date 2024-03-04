use ratatui::{layout::Rect, Frame};

use crate::app::state::AppState;
use crate::kubernetes::api::pod::PodDescribe;

pub fn draw_pod_resource(
    f: &mut Frame,
    state: &AppState,
    pod_describe: Option<&PodDescribe>,
    area: Rect,
) {
    if pod_describe.is_none() {
        return;
    }
    let pod_describe = pod_describe.unwrap();
    let namespace: &str;
    let pod_name: &str;
    unsafe {
        namespace = &(*pod_describe.namespace);
        pod_name = &(*pod_describe.name);
    }

    f.render_widget(state.metrics_buffer.widget(), area);
    if let Some(_namespaced_metrics) = state.pod_metrics_cache.get(namespace) {
        if let Some(_pod_metrics) = _namespaced_metrics.get(pod_name) {}
    }
}
