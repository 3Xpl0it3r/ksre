use std::borrow::BorrowMut;
use std::fmt::format;
use std::rc::Rc;

use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
// pod ui
//
//
//
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::action::{Mode, RouteId};
use crate::app::state::AppState;
use crate::kubernetes::api::{ContainerFields, PodFields};

use super::color;
use super::util::{
    self, debug_widget, selectable_list, selectable_list_with_filter, selectable_tabpages,
    titled_block, user_input, vertical_chunks, vertical_margined_chunks,
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

pub(super) fn draw(f: &mut Frame, state: &mut AppState, area: Rect) {
    let chunks = util::vertical_chunks(
        vec![Constraint::Percentage(70), Constraint::Percentage(30)],
        area,
    );
    let pod_chunks = util::horizontal_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        chunks[0],
    );
    let pod_fields = draw_pod_list(f, pod_chunks[0], state);

    // delivered ownership
    let pod_fields = draw_pod_info(f, pod_chunks[1], pod_fields);

    draw_pod_debug(f, chunks[1], state, pod_fields);
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
    if pod_name.is_some() {
        if let Some(obj) = state.pod_cache.get_value(pod_name.unwrap()) {
            let pod_fiels: PodFields = obj.into();
            return Some(pod_fiels);
        }
    }

    None
}

fn draw_pod_info(
    f: &mut Frame,
    area: Rect,
    /* state: &mut AppState, */
    pod_fields: Option<PodFields>,
) -> Option<PodFields> {
    if pod_fields.is_none() {
        f.render_widget(debug_widget("".to_owned()), area);
        return None;
    }
    let pod_fields = pod_fields.unwrap();

    let mut ver_mess: Vec<String> = vec![
        format!("nodename:\t{}", pod_fields.node_name),
        format!("service_account:\t{}", pod_fields.service_account),
    ];
    for container in &pod_fields.containers {
        ver_mess.push(format!(
            "[{}]\nliveness:\t{}\nreadneess:\t{}",
            container.name,
            container
                .liveness_probe
                .as_ref()
                .unwrap_or(&"notfound".to_string()),
            &container
                .readness_probe
                .as_ref()
                .unwrap_or(&"no found".to_string())
        ));
    }

    f.render_widget(
        Paragraph::new(ver_mess.join("\n")).block(
            Block::default()
                .title("Pod Info")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        area,
    );
    Some(pod_fields)
}

fn draw_pod_debug(f: &mut Frame, area: Rect, state: &mut AppState, pod_fields: Option<PodFields>) {
    if pod_fields.is_none() {
        f.render_widget(debug_widget("".to_owned()), area);
        return;
    }
    let area = vertical_chunks(vec![Constraint::Length(3), Constraint::Min(0)], area);
    let tabs = selectable_tabpages(vec!["status", "logs", "terminal"], 0);

    f.render_widget(tabs, area[0]);
    match state.id_cur_route {
        RouteId::PodLog => to_pod_status(f, area[1], state, pod_fields),
        RouteId::PodTerminal => {}
        RouteId::PodYaml => {}
        // default show pod status
        _ => {}
    }
    /* f.render_widget(debug_widget(msg), area[1]); */
}

// drawp log_info, pod logs, pod_exec
fn to_pod_status(f: &mut Frame, area: Rect, status: &mut AppState, pod_fields: Option<PodFields>) {
    let msg = format!(
        "
pod_ip: {},

                      ",
        pod_fields.as_ref().unwrap().node_name
    );
    f.render_widget(debug_widget(msg.to_string()), area)
}

fn to_pod_logs(f: &mut Frame, area: Rect, status: &mut AppState, pod_fields: Option<PodFields>) {
}

fn to_pod_terminal(
    f: &mut Frame,
    area: Rect,
    status: &mut AppState,
    pod_fields: Option<PodFields>,
) {
}
