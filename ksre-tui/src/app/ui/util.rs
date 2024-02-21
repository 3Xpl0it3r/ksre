use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Styled, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::action::Mode;
use crate::app::state::StatefulList;
use crate::app::ui::theme;

use super::theme::Kanagawa;

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
    constraints: Vec<Constraint>,
    size: Rect,
    margin: u16,
) -> Rc<[Rect]> {
    Layout::default()
        .constraints(constraints)
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
                .title("Select pod, [e]:trigger [esc]:exit input")
                .border_type(BorderType::Rounded),
        )
}

pub(super) fn selected_tab(values: Vec<&str>, id_selected: usize) -> Tabs<'_> {
    let colored_items = values
        .iter()
        .map(|x| x.to_string().bg(theme::DefaultTheme::Sumlink1).into())
        .collect::<Vec<Line>>();
    Tabs::new(colored_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .fg(theme::DefaultTheme::OrangeSurimi)
                .bg(theme::DefaultTheme::Sumlink1),
        )
        .select(id_selected)
        .divider(" ")
        .padding(" ", "")
}

pub(super) fn selectable_list_0(stateful_list: &StatefulList) -> List {
    let mut list_items = Vec::new();
    let items = stateful_list.items.iter();

    for (idx, val) in items.enumerate() {
        if idx == stateful_list.index {
            list_items
                .push(ListItem::new(val.as_ref()).style(Style::default().fg(theme::DefaultTheme::BlueSpring).bg(theme::DefaultTheme::Sumlink1)));
        } else {
            list_items.push(ListItem::new(val.as_ref()).style(Style::default()));
        }
    }
    List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

pub(super) fn selectable_list_with_mark(stateful_list: &StatefulList) -> List {
    let mut list_items = Vec::new();
    let items = stateful_list.items.iter();

    for (idx, val) in items.enumerate() {
        if idx == stateful_list.index {
            if stateful_list.fixed {
                list_items.push(ListItem::new(Line::styled(
                    format!("[✓] {}", val.as_ref()),
                    Style::default(),
                )));
            } else {
                list_items.push(ListItem::new(Line::styled(
                    format!("[*] {}", val.as_ref()),
                    Style::default(),
                )));
            }
        } else {
            list_items.push(ListItem::new(Line::styled(
                format!("[ ] {}", val.as_ref()),
                Style::default(),
            )));
        }
    }
    List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

pub(super) fn debug_widget(msg: &str) -> Paragraph {
    let msg_wideget = Paragraph::new(msg).block(
        Block::default()
            .borders(Borders::ALL)
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

