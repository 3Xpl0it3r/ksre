use ratatui::Frame;

use ratatui::layout::{Layout, Rect};

use crate::app::AppState;

use super::util::{titled_block, self};

pub(super) fn draw_main(f: &mut Frame, area: Rect, state: &mut AppState) {

    f.render_widget(util::titled_block("nihao"), area);
}
