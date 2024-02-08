use ratatui::Frame;

use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Constraint;
use ratatui::widgets::Tabs;

use crate::app::AppState;

use super::util::{self, vertical_chunks, titled_block};

pub(super) fn draw_main(f: &mut Frame, size: Rect, state: &mut AppState) {
    let chunks = util::vertical_chunks(vec![Constraint::Length(3), Constraint::Min(1)], f.size());
    draw_header(f, chunks[0], state.cur_tab as usize);
}
fn draw_header(f: &mut Frame, area: Rect, tab_nr: usize) {
    f.render_widget(util::titled_block("hehe"), area);
}
