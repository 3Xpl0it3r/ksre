use std::usize;

use ratatui::layout::{Constraint, Rect};

use ratatui::Frame;
use tui_textarea::TextArea;

use super::{pod, util as uiutil};
use crate::app::state::{AppState, TabPage};

const HEAD_TITLE: &'_ str = "ksre - ksre tools";

pub enum View {
    Pod,
    Deploy,
    Node,
    Error,
}

pub fn ui_main(
    f: &mut Frame,
    state: &mut AppState,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let chunks = uiutil::vertical_chunks(vec![Constraint::Length(3), Constraint::Min(1)], f.size());
    // header  pods  nodes
    draw_header(f, chunks[0], state.get_tabpage() as usize);

    // pod index is default home page
    match state.get_tabpage() {
        TabPage::Pod => pod::draw_page_index(f, state, chunks[1], reader),
        TabPage::Deploy => todo_fn(),
        TabPage::Node => todo_fn(),
    }
}
fn todo_fn() {}

fn draw_header(f: &mut Frame, area: Rect, tab_nr: usize) {
    f.render_widget(uiutil::titled_block(HEAD_TITLE), area);

    let tabs = uiutil::selected_tab(vec!["[ pods ]", "[ deployment ]", "[ nodes ]"], tab_nr);

    f.render_widget(tabs, area);

    /* let chunks =
        util::horizontal_margined_chunks(vec![Constraint::Length(75), Constraint::Min(0)], area, 1);
    let tabs = Tabs::new(vec!["Workload <tab|0>", "Cluster <tab|1>", "Node <tab|2>"])
        .highlight_style(Style::default().fg(Color::from_u32(color::Green)))
        .select(tab_nr);

    f.render_widget(tabs, chunks[0]) */
}
