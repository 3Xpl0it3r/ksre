use std::rc::Rc;

use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};
use tui_textarea::TextArea;

use crate::app::ui::util::{self as uituil, no_border_windows};
use crate::app::AppState;
use crate::kubernetes::api::PodFields;

pub fn draw_pod_logs(
    f: &mut Frame,
    state: &mut AppState,
    pod_fields: Option<&PodFields>,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let outer = uituil::outer_block(f, "Log [esc to quit]", area);

    f.render_widget(reader.widget(), outer);
}

