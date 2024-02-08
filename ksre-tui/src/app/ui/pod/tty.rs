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

    if !state.input_char.is_empty(){
        show_user_commnad(f, 0, outer.y, state.input_char.as_str());
    }

    let message = reader.join("\n").to_string();
    f.render_widget(debug_widget(message, " ".to_string()), outer);

}

fn show_user_commnad(f: &mut Frame, x: u16, y: u16, cmd: &str) {
    let area = Rect::new(1, y - 4, 32, 3);
    let txt = debug_widget(cmd.to_string(), "command".to_string());
    f.render_widget(txt, area);
}