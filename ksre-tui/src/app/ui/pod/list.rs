use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::List;
use ratatui::Frame;

use crate::app::action::{Mode, Route};
use crate::app::state::AppState;
use crate::kubernetes::api::PodFields;

use crate::app::ui::util::{self as uiutil, debug_widget};

// filter <e toggle>
// namespace<n toggle> | nodename<N toogle> |
// podlist

// page for show pod list
pub fn draw_page_pod_list(f: &mut Frame, area: Rect, state: &mut AppState) {
    // split windows chunks[0] for input, chunk[1] for podlist
    let area = uiutil::vertical_chunks(
        vec![
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(3),
        ],
        area,
    );
    let input_area = area[0];
    let ns_select_area = area[1];
    let pod_list_area = area[2];

    // draw user input
    draw_user_input(f, input_area, state);

    ui_select_namespace(f, ns_select_area, state);

    ui_select_pod(f, pod_list_area, state);
    /* let input = state.input_char.to_string(); */

    /* let listitems = uiutil::selectable_list(&state.cache_items); */
}

fn draw_user_input(f: &mut Frame, area: Rect, state: &AppState) {
    let input_widget = if let Route::PodList = state.cur_route {
        uiutil::user_input(state.user_input.as_str(), state.cur_mode)
    } else {
        uiutil::user_input("", Mode::Normal)
    };
    f.render_widget(input_widget, area);
}

fn ui_select_namespace(f: &mut Frame, area: Rect, state: &AppState) {
    let list = uiutil::selectable_list_with_mark(&state.namespace_items);

    f.render_widget(list, area);
}


fn ui_select_pod(f: &mut Frame, area: Rect, state: &AppState) {
    let list = uiutil::selectable_list_0(&state.cache_items);
    f.render_widget(list, area);
}
