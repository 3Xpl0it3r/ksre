use std::usize;

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use crate::app::action::RouteId;
use crate::app::state::AppState;

use super::color;
use super::pod;
use super::util;

const HEAD_TITLE: &'_ str = "ksre - ksre tools";

pub enum View {
    Pod,
    Deploy,
    Node,
    Error,
}

// From[#TODO] (should add some comments)
impl From<RouteId> for View {
    fn from(value: RouteId) -> Self {
        if value >= RouteId::PodIndex && value <= RouteId::PodEnd {
            View::Pod
        } else if value >= RouteId::DeployIndex && value <= RouteId::DeployEnd {
            View::Deploy
        } else if value >= RouteId::NodeIndex && value <= RouteId::NodeEnd {
            View::Node
        } else {
            View::Error
        }
    }
}

pub fn ui_main(f: &mut Frame, state: &mut AppState) {
    let chunks = util::vertical_chunks(vec![Constraint::Length(3), Constraint::Min(1)], f.size());
    draw_header(f, chunks[0], state.id_cur_route as usize);

    match state.id_cur_route.into() {
        View::Pod => pod::draw(f, state, chunks[1]),
        View::Deploy => todo_fn(),
        View::Node => todo_fn(),
        View::Error => unreachable!(),
    }
}
fn todo_fn() {}

fn draw_header(f: &mut Frame, area: Rect, tab_nr: usize) {
    f.render_widget(util::titled_block(HEAD_TITLE), area);

    let chunks =
        util::horizontal_margined_chunks(vec![Constraint::Length(75), Constraint::Min(0)], area, 1);
    let tabs = Tabs::new(vec!["Workload <tab|0>", "Cluster <tab|1>", "Node <tab|2>"])
        .highlight_style(Style::default().fg(Color::from_u32(color::Green)))
        .select(tab_nr);

    f.render_widget(tabs, chunks[0])
}
