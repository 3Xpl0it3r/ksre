use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;

use crate::app::action::Route;
use crate::app::state::AppState;

use crate::app::ui::{
    pod::{draw_page_pod_spec, draw_page_pod_status, draw_page_pod_tty,draw_page_pod_list},
    util,
};

pub fn draw_page_index(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    reader: tokio::sync::RwLockReadGuard<Vec<String>>,
) {
    let chunks = util::vertical_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        area,
    );

    // pod_chunks[0] for podlist, pod_chunks【1】 is used to render pod status
    let pod_chunks = util::horizontal_chunks(
        vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        chunks[0],
    );

    // pod_chunks[0] 展示pod list, ,pod_chunks[1] 展示 pod status
    let pod_fields = draw_page_pod_list(f, pod_chunks[0], state);
    let pod_fields = draw_page_pod_status(f, state, pod_fields, pod_chunks[1]);

    match state.cur_route {
        Route::PodTerm => draw_page_pod_tty(f, chunks[1], state, reader),
        _ => draw_page_pod_spec(f, state, pod_fields, chunks[1]),
    }
}
