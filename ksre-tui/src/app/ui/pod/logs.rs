use std::rc::Rc;

use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};
use tui_textarea::TextArea;

use crate::app::ui::util::{self as uituil, debug_widget, no_border_windows};
use crate::app::AppState;
use crate::kubernetes::api::PodDescribe;

pub fn draw_pod_logs(
    f: &mut Frame,
    state: &mut AppState,
    pod_fields: Option<&PodDescribe>,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let full_name = state.get_namespaced_pod();
    if full_name.is_none() {
        let outer = uituil::outer_block(f, "Log [esc to quit]", area);
        return;
    }
    let (namespace, pod_name) = full_name.unwrap();
    let outer = uituil::outer_block(f, format!("show {}:{} log [esc]:quit", namespace, pod_name).as_str(), area);

    f.render_widget(reader.widget(), outer);
}
