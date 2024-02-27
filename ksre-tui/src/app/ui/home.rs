use std::usize;

use ratatui::layout::{Constraint, Rect};


use ratatui::Frame;
use tui_textarea::TextArea;

use crate::app::action::Route;
use crate::app::state::AppState;

use super::pod;
use crate::app::ui::util as uiutil;

const HEAD_TITLE: &'_ str = "ksre - ksre tools";

pub enum View {
    Pod,
    Deploy,
    Node,
    Error,
}

// From[#TODO] (should add some comments)
impl From<Route> for View {
    fn from(value: Route) -> Self {
        if value >= Route::PodIndex && value <= Route::PodEnd {
            View::Pod
        } else if value >= Route::DeployIndex && value <= Route::DeployEnd {
            View::Deploy
        } else if value >= Route::NodeIndex && value <= Route::NodeEnd {
            View::Node
        } else {
            View::Error
        }
    }
}

pub fn ui_main(
    f: &mut Frame,
    state: &mut AppState,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let chunks = uiutil::vertical_chunks(vec![Constraint::Length(3), Constraint::Min(1)], f.size());
    // header  pods  nodes
    draw_header(f, chunks[0], state.cur_route as usize);

    // pod index is default home page
    match state.cur_route.into() {
        View::Pod => pod::draw_page_index(f, state, chunks[1], reader),
        View::Deploy => todo_fn(),
        View::Node => todo_fn(),
        View::Error => unreachable!(),
    }
}
fn todo_fn() {}

fn draw_header(f: &mut Frame, area: Rect, tab_nr: usize) {
    let tab_nr = tab_nr / Route::route_step() as usize;
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
