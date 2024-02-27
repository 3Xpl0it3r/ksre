use ratatui::layout::{Rect};
use ratatui::Frame;

use crate::app::state::AppState;


use crate::app::ui::util::{self as uiutil, debug_widget};

pub fn draw_page_pod_tty(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let outer = uiutil::outer_block(f, "Terminal", area);

    if !state.user_input.is_empty(){
        show_user_commnad(f, 0, outer.y, state.user_input.as_str());
    }

    let message = reader.join("\n");
    f.render_widget(debug_widget(message.as_str()), outer);

}

fn show_user_commnad(f: &mut Frame, _x: u16, y: u16, cmd: &str) {
    let area = Rect::new(1, y - 4, 32, 3);
    let txt = debug_widget(cmd);
    f.render_widget(txt, area);
}
