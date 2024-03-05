use ratatui::style::{Style, Stylize};
use ratatui::{
    layout::{Constraint, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};
use tui_textarea::TextArea;

use crate::app::state::Route;
use crate::app::{
    state::AppState,
    ui::{
        theme::{self, Kanagawa},
        util::{self as uiutil, horizontal_chunks, outer_block, vertical_chunks},
    },
};

use super::{
    list::draw_page_pod_list, logs::draw_pod_logs, resource::draw_pod_resource,
    status::draw_page_pod_status,
};

// -------------------------------------
// ---- pods          | resource       |
// [namespace] [node] | resource       |
// [pod list]         | cpu memory net |
// ------------------------------------|
// devops
// logs| exec

pub fn draw_page_index(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<TextArea>,
) {
    let chunks = uiutil::vertical_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        area,
    );

    // pod_chunks[0] for pod list, pod_chunks【1】 is used to render pod status
    let main_area = uiutil::horizontal_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        chunks[0],
    );

    let pod_list_area = uiutil::outer_block(f, "Pods", main_area[0]);
    let pod_res_area = uiutil::outer_block(f, "Pod Resource", main_area[1]);

    let bottom_area = vertical_chunks(vec![Constraint::Length(3), Constraint::Min(2)], chunks[1]);
    let bottom_head = bottom_area[0];
    let bottom_body = bottom_area[1];

    // pod_chunks[0] 展示pod list, ,pod_chunks[1] 展示 pod status
    //
    draw_page_pod_list(f, pod_list_area, state);

    let namespace = state.namespace_cache.get().unwrap();

    // devops split 2 items
    draw_bottom_head(f, state, bottom_head);

    if let Some(pod) = state.cache_items.get() {
        let pod_describe = state.pod_describes.get(namespace.as_ref(), pod.as_ref());
        draw_pod_resource(f, state, pod_describe, pod_res_area);

        match state.get_route() {
            Route::PodLog => draw_pod_logs(f, state, pod_describe, bottom_body, reader),
            Route::PodTerm => {}
            _ => draw_page_pod_status(f, state, pod_describe, bottom_body),
        }
        return;
    }
    draw_pod_resource(f, state, None, pod_res_area);
    match state.get_route() {
        Route::PodLog => draw_pod_logs(f, state, None, bottom_body, reader),
        Route::PodTerm => {}
        _ => draw_page_pod_status(f, state, None, bottom_body),
    }
}

fn draw_bottom_head(f: &mut Frame, state: &AppState, area: Rect) {
    let area = outer_block(f, "", area);
    let area = horizontal_chunks(
        vec![Constraint::Percentage(20), Constraint::Percentage(50)],
        area,
    );
    let id_selected = match state.get_route() {
        Route::PodLog => 1,
        Route::PodTerm => 2,
        _ => 0,
    };
    let colored_items = vec!["Describe", "Log", "Terminal"]
        .iter()
        .map(|&x| x.to_string().bg(theme::DefaultTheme::SUMLINK1).into())
        .collect::<Vec<Line>>();

    let tabs = Tabs::new(colored_items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().fg(theme::DefaultTheme::VIOLET_ONI))
        .select(id_selected)
        .divider(" ")
        .padding(" ", "");

    f.render_widget(tabs, area[0]);

    let help_message =
        r#"help: [l]:show pods log, [t]:attach pod, [esc] exit then reback to descibe"#;
    f.render_widget(Paragraph::new(help_message), area[1]);
}
