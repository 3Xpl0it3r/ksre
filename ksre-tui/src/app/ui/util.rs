use std::rc::Rc;

use k8s_openapi::api::core::v1::Pod;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::Matcher;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use super::color;
use crate::app::action::Mode;
use crate::app::state::StatefulList;

pub(super) fn style_error() -> Style {
    Style::default().fg(Color::Red)
}

pub(super) fn style_warn() -> Style {
    Style::default().fg(Color::Yellow)
}
pub(super) fn style_info() -> Style {
    Style::default().fg(Color::Green)
}

pub(super) fn titled_block(title: &'static str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

pub(super) fn vertical_chunks(constraits: Vec<Constraint>, size: Rect) -> Rc<[Rect]> {
    Layout::default()
        .constraints(constraits)
        .direction(Direction::Vertical)
        .split(size)
}

pub(super) fn vertical_margined_chunks(
    constraits: Vec<Constraint>,
    size: Rect,
    margin: u16,
) -> Rc<[Rect]> {
    Layout::default()
        .constraints(constraits)
        .direction(Direction::Vertical)
        .margin(margin)
        .split(size)
}

pub(super) fn horizontal_chunks(constraits: Vec<Constraint>, size: Rect) -> Rc<[Rect]> {
    Layout::default()
        .constraints(constraits)
        .direction(Direction::Horizontal)
        .split(size)
}

pub(super) fn horizontal_margined_chunks(
    constraits: Vec<Constraint>,
    size: Rect,
    margin: u16,
) -> Rc<[Rect]> {
    Layout::default()
        .constraints(constraits)
        .direction(Direction::Horizontal)
        .margin(margin)
        .split(size)
}

pub(super) fn user_input(input_char: &'_ str, input_mode: Mode) -> Paragraph {
    Paragraph::new(input_char)
        .style(match input_mode {
            Mode::Normal => Style::default(),
            Mode::Insert => Style::default().fg(Color::LightYellow),
            Mode::Command => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input Your Command")
                .border_type(BorderType::Rounded),
        )
}

pub(super) fn selectable_tabpages(values: Vec<&str>, id_selected: usize) -> Tabs<'_> {
    Tabs::new(values)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().fg(Color::from_u32(color::Green)))
        .select(id_selected)
}

pub(super) fn selectable_list<'a>(stateful_list: &'a StatefulList) -> List<'a> {
    let mut list_items = Vec::new();
    let items = stateful_list.items.iter();

    let sub_items = &stateful_list.sub_items;
    for (idx, val) in items.enumerate() {
        if idx == stateful_list.index {
            list_items
                .push(ListItem::new(val.as_str()).style(Style::default().fg(Color::LightYellow)));
        } else {
            list_items.push(ListItem::new(val.as_str()).style(Style::default()));
        }
        if sub_items.get(&idx).is_none() {
            continue;
        }
        for sub_item in sub_items.get(&idx).unwrap().iter() {
            list_items.push(ListItem::new(format!("    |-{}", sub_item)).style(Style::default()));
        }
    }
    List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

pub(super) fn selectable_list_with_filter<'a>(
    stateful_list: &'a StatefulList,
    input: &str,
    matcher: &mut Matcher,
) -> List<'a> {
    let mut list_items = Vec::new();
    let items = stateful_list.items.iter();
    let filterd_items = Atom::new(
        input,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    )
    .match_list(items, matcher)
    .iter()
    .map(|x| x.0)
    .collect::<Vec<&String>>();

    for (idx, val) in filterd_items.into_iter().enumerate() {
        list_items.push(ListItem::new(val.as_str()).style(Style::default()));
    }

    List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

pub(super) fn debug_widget(msg: String, title: String) -> Paragraph<'static> {
    let msg_wideget = Paragraph::new(msg).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title),
    );
    msg_wideget
}

pub(super) fn no_border_windows(msg: String) -> Paragraph<'static> {
    let msg_wideget = Paragraph::new(msg).block(
        Block::default()
            .borders(Borders::NONE)
            .border_type(BorderType::Rounded),
    );
    msg_wideget
}

pub(super) fn outer_block(f: &mut Frame, title: &str, area: Rect) -> Rect {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default())
        .title(title);
    f.render_widget(outer, area);
    Rect::new(area.x + 1, area.y + 1, area.width - 1, area.height - 1)
}
