use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};

use crate::app::ui::util::{self as uituil, no_border_windows};
use crate::app::AppState;
use crate::kubernetes::api::PodFields;

pub fn draw_page_pod_status(
    f: &mut Frame,
    state: &mut AppState,
    pod_fields: Option<PodFields>,
    area: Rect,
) -> Option<PodFields> {
    let outer = uituil::outer_block(f, "Pod Status", area);

    pod_fields.as_ref()?;

    let pod_fields = pod_fields.unwrap();

    // split area into nums according the number of containers
    //
    // account the number of containers
    let container_nb = pod_fields.container_status.len() as u16;

    let mut constrats = Vec::new();
    for _ in 0..container_nb {
        constrats.push(Constraint::Percentage(100 / container_nb));
    }

    let container_area = uituil::vertical_margined_chunks(constrats, outer, 1);

    for (idx, cs) in pod_fields.container_status.iter().enumerate() {
        // split windows into  | container_status:|     value   | ok|
        let container_outer = uituil::outer_block(
            f,
            format!("Container-{}: [{}]", idx, cs.name.as_str()).as_str(),
            container_area[idx],
        );
        let cs_area = uituil::horizontal_chunks(
            vec![Constraint::Length(16), Constraint::Min(4)],
            container_outer,
        );
        let mut keys = Vec::new();
        let mut value = Vec::new();

        if cs.ready {
            keys.push("[✔] Ready:");
            value.push("Not Ready:")
        } else {
            keys.push("[✘] Ready:");
            value.push("Ready");
        }

        if cs.restart_count.eq("0") {
            keys.push("[✔] Restart:");
            value.push("0");
        } else {
            keys.push("[✘] Restart:");
            value.push(cs.restart_count.as_str());
        }
        keys.push("[✔] state:");
        value.push(cs.state.as_str());
        keys.push("[✔] exit_code");
        value.push(cs.exit_code.as_str());
        keys.push("[✔] message:");
        value.push(cs.message.as_str());
        keys.push("[✔] reason:");
        value.push(cs.reason.as_str());
        keys.push("[✔] signal");
        value.push(cs.signal.as_str());

        f.render_widget(no_border_windows(keys.join("\n").to_string()), cs_area[0]);
        f.render_widget(no_border_windows(value.join("\n").to_string()), cs_area[1]);
    }
    Some(pod_fields)
}

// legy
/* fn styple_text(txt: &str, alig: Alignment, style: Style) -> Text<'static> {
    Text::from(txt).style(style).alignment(alig)
} */
