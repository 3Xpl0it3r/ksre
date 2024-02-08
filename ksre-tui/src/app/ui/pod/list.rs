use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::List;
use ratatui::Frame;

use crate::app::action::{Mode, Route};
use crate::app::state::AppState;
use crate::kubernetes::api::PodFields;

use crate::app::ui::util::{self as uiutil, debug_widget};

// page for show pod list
pub fn draw_page_pod_list(f: &mut Frame, area: Rect, state: &mut AppState) -> Option<PodFields> {
    // split windows chunks[0] for input, chunk[1] for podlist
    let area = uiutil::vertical_chunks(vec![Constraint::Fixed(3), Constraint::Min(10)], area);

    /* let input = state.input_char.to_string(); */
    let input_widget = if let Route::PodList = state.cur_route {
        uiutil::user_input(state.input_char.as_str(), state.cur_mode)
    } else {
        uiutil::user_input("", Mode::Normal)
    };
    f.render_widget(input_widget, area[0]);

    // get selected items
    if state.cur_route == Route::PodList && !state.input_char.is_empty() {
        let listitems = uiutil::selectable_list_with_filter(
            &state.cache_items,
            state.input_char.as_str(),
            &mut state.fuzz_matcher,
        );
        f.render_widget(listitems, area[1]);
    } else {
        let listitems = uiutil::selectable_list(&state.cache_items);
        f.render_widget(listitems, area[1]);
    }

    /* let listitems = match state.input_char.len() {
        0 => uiutil::selectable_list(&state.cache_items),
        _ => uiutil::selectable_list_with_filter(&state.cache_items, || true),
    }; */

    // get podfields to render the following widgets
    let pod_name = state.cache_items.items.get(state.cache_items.index);
    if let Some(pod_name) = pod_name {
        if let Some(obj) = state.store_pods.get_value(pod_name) {
            let pod_fiels: PodFields = obj.into();
            return Some(pod_fiels);
        }
    }

    None
}
