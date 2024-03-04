use ratatui::Frame;

use ratatui::layout::Rect;
use tui_textarea::TextArea;

use crate::app::{
    state::AppState,
    ui::util::{self as uituil},
};
use crate::kubernetes::api::pod::PodDescribe;

pub fn draw_pod_logs(
    f: &mut Frame,
    state: &AppState,
    _pod_fields: Option<&PodDescribe>,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let namespace = state.namespace_cache.get().unwrap();
    if let Some(pod_name) = state.cache_items.get() {
        let outer = uituil::outer_block(
            f,
            format!("show {}:{} log [esc]:quit", namespace, pod_name).as_str(),
            area,
        );
        f.render_widget(reader.widget(), outer);
    } else {
        let outer = uituil::outer_block(f, "Log [esc to quit]", area);
        f.render_widget(reader.widget(), outer);
    }
}
