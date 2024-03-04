use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::state::AppState;

use crate::app::ui::util::{self as uiutil, debug_widget};

#[allow(dead_code)]
pub fn draw_page_pod_tty(
    f: &mut Frame,
    area: Rect,
    _state: &mut AppState,
    _reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let _outer = uiutil::outer_block(f, "Terminal", area);
}

#[allow(dead_code)]
fn show_user_commnad(f: &mut Frame, _x: u16, y: u16, cmd: &str) {
    let area = Rect::new(1, y - 4, 32, 3);
    let txt = debug_widget(cmd);
    f.render_widget(txt, area);
}
