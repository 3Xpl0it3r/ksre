use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};

use crate::app::ui::util::{self as uituil, no_border_windows};
use crate::app::AppState;
use crate::kubernetes::api::PodFields;

pub fn draw_pod_resource(
    f: &mut Frame,
    state: &mut AppState,
    pod_fields: Option<&PodFields>,
    area: Rect,
) {
    let outer = uituil::outer_block(f, "Pod Top", area);

    if pod_fields.is_none() {
        return;
    }
}

// legy
/* fn styple_text(txt: &str, alig: Alignment, style: Style) -> Text<'static> {
    Text::from(txt).style(style).alignment(alig)
} */
