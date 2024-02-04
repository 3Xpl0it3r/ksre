use std::borrow::BorrowMut;
use std::fmt::format;
use std::rc::Rc;

use color_eyre::owo_colors::OwoColorize;
use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use ratatui::prelude::Alignment;
// pod ui
//
//
//
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{
    block, Block, BorderType, Borders, List, ListItem, Paragraph, Tabs, Widget,
};
use ratatui::Frame;

use crate::app::action::{Mode, RouteId};
use crate::app::state::AppState;
use crate::kubernetes::api::{ContainerFields, PodFields};

use super::color;
use super::util::{
    self, debug_widget, horizontal_chunks, outer_block, selectable_list,
    selectable_list_with_filter, selectable_tabpages, style_info, titled_block, user_input,
    vertical_chunks, vertical_margined_chunks,
};

///
/// |pod list|
/// |pod1|  nodename:
/// |pod2|  pod:io
/// |pod3|  resourcelimit
/// |pod4|  liveness: xxxx
/// /pod5|  readness: xxx
/// |pod6|  cpu: request: xxx, limit:xxx
/// |pod7|  mem: request:xxx limit:xxx
/// |pod7|  lastRestart: xxx
/// ------DEBUG-------------
/// logs | exec | desc | status
/// NodeCheck: ok
/// diskCheck: ok
/// xxx check: ok
///

pub(super) fn draw(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let chunks = util::vertical_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        area,
    );
    let pod_chunks = util::horizontal_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        chunks[0],
    );
    let pod_fields = draw_pod_list(f, pod_chunks[0], state);

    // delivered ownership
    let pod_fields = draw_pod_spec(f, pod_chunks[1], pod_fields);

    draw_pod_debug(f, chunks[1], state, pod_fields, reader);
}

/// return value is used to derlived podfields owner ship
fn draw_pod_list(f: &mut Frame, area: Rect, state: &mut AppState) -> Option<PodFields> {
    let current_select_id = state.select_items.index;

    let area = vertical_chunks(vec![Constraint::Fixed(3), Constraint::Min(10)], area);

    // get selected items
    let listitems = match state.input_char.len() {
        0 => selectable_list(&state.select_items),
        _ => selectable_list_with_filter(&state.select_items, || true),
    };

    f.render_widget(
        user_input(state.input_char.as_str(), state.op_mode),
        area[0],
    );
    f.render_widget(listitems, area[1]);

    // get podfields to render the following widgets
    let pod_name = state.select_items.items.get(state.select_items.index);
    if let Some(pod_name) = pod_name {
        if let Some(obj) = state.cache_pods.get_value(&pod_name) {
            let pod_fiels: PodFields = obj.into();
            return Some(pod_fiels);
        }
    }

    None
}

fn draw_pod_spec(
    f: &mut Frame,
    area: Rect,
    /* state: &mut AppState, */
    pod_fields: Option<PodFields>,
) -> Option<PodFields> {
    let area = outer_block(f, "Pod Info", area);
    if pod_fields.is_none() {
        return None;
    }

    let pod_fields = pod_fields.unwrap();

    let win_num = pod_fields.containers.len();
    let mut constraint_vec = Vec::new();
    for i in 0..win_num {
        constraint_vec.push(Constraint::Percentage((100 / win_num) as u16));
    }
    let area = horizontal_chunks(constraint_vec, area);
    for (id, container_field) in pod_fields.containers.iter().enumerate() {
        let mut info_items = Vec::new();
        info_items.push(format!(
            "{:<24}\t{:>}",
            "cpu_request",
            container_field
                .cpu_request
                .as_ref()
                .unwrap_or(&"not set".to_string())
                .to_string(),
        ));
        info_items.push(format!(
            "{:<24}\t{:>}",
            "cpu_limit".to_string(),
            container_field
                .cpu_limit
                .as_ref()
                .unwrap_or(&"no set".to_string())
                .to_string(),
        ));
        info_items.push(format!(
            "{:<24}\t{:>}",
            "mem_request".to_string(),
            container_field
                .mem_requrst
                .as_ref()
                .unwrap_or(&"no set".to_string())
                .to_string(),
        ));
        info_items.push(format!(
            "{:<24}\t{:>}",
            "mem_limit".to_string(),
            container_field
                .mem_limit
                .as_ref()
                .unwrap_or(&"no set".to_string())
                .to_string(),
        ));
        info_items.push(format!(
            "{:<24}\n    {:<}",
            "image".to_string(),
            container_field.image.to_owned(),
        ));
        info_items.push(format!(
            "{:<24}\t{:>}",
            "liveness_probe".to_string(),
            container_field
                .liveness_probe
                .as_ref()
                .unwrap_or(&"no set".to_string())
                .to_string(),
        ));
        info_items.push(format!(
            "{:<24}:\t{:>}",
            "readness_probe".to_string(),
            container_field
                .readness_probe
                .as_ref()
                .unwrap_or(&"no set".to_string())
                .to_string(),
        ));
        f.render_widget(
            debug_widget(info_items.join("\n"), "<Container> : <Heheh>".to_string()),
            area[id],
        );
    }
    Some(pod_fields)
}

fn draw_pod_debug(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    pod_fields: Option<PodFields>,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    if pod_fields.is_none() {
        f.render_widget(debug_widget("".to_owned(), "debug".into()), area);
        return;
    }
    let area = vertical_chunks(vec![Constraint::Length(3), Constraint::Min(0)], area);
    let tabs = selectable_tabpages(vec!["status [s]", "logs [l]", "terminal [t]"], 0);

    f.render_widget(tabs, area[0]);
    match state.id_cur_route {
        RouteId::PodLog => to_pod_logs(f, area[1], state, reader),
        _ => to_pod_logs(f, area[1], state, reader),
        // default show pod status
    }
    /* f.render_widget(debug_widget(msg), area[1]); */
}

// drawp log_info, pod logs, pod_exec
fn to_pod_status(f: &mut Frame, area: Rect, status: &mut AppState, pod_fields: Option<PodFields>) {
    let area = outer_block(f, "Pod Info", area);
}

fn to_pod_logs(
    f: &mut Frame,
    area: Rect,
    status: &mut AppState,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let messa = reader.join("\n");
    let pg = debug_widget(messa, "pod logs".to_string());

    f.render_widget(pg, area)
}

fn to_pod_terminal(
    f: &mut Frame,
    area: Rect,
    status: &mut AppState,
    pod_fields: Option<PodFields>,
) {
}
