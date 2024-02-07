use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::kubernetes::api::PodFields;

use crate::app::ui::util::{self as uiutil, debug_widget};

pub fn draw_page_pod_tty(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let outer = uiutil::outer_block(f, "Terminal", area);

    let message = reader.join("\n").to_string();

    f.render_widget(debug_widget(message, " ".to_string()), outer);

}
