use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};

use crate::app::ui::util::{self as uituil, debug_widget, no_border_windows};
use crate::app::AppState;
use crate::kubernetes::api::PodFields;

pub fn draw_pod_resource(
    f: &mut Frame,
    state: & AppState,
    pod_fields: Option<&PodFields>,
    area: Rect,
) {
    let outer = uituil::outer_block(f, "Pod Top", area);

    let message = format!(
        "cur route: {:?}, mode: {:?}, input: {:?}",
        state.cur_route, state.cur_mode, state.user_input.clone(),
    );

    f.render_widget(debug_widget(message, "hehe".to_string()), outer);
}

// legy
/* fn styple_text(txt: &str, alig: Alignment, style: Style) -> Text<'static> {
    Text::from(txt).style(style).alignment(alig)
} */
