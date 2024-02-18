use ratatui::Frame;

use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::AppState;
use crate::kubernetes::api::PodDescribe;

pub fn draw_pod_resource(
    f: &mut Frame,
    state: &AppState,
    pod_fields: Option<&PodDescribe>,
    area: Rect,
) {
    let message = r#"TODO (metric from metric-server)"#;

    f.render_widget(debug_widget(message), area);
}

// legy
/* fn styple_text(txt: &str, alig: Alignment, style: Style) -> Text<'static> {
    Text::from(txt).style(style).alignment(alig)
} */
fn debug_widget(msg: &str) -> Paragraph {
    let msg_wideget = Paragraph::new(msg).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    msg_wideget
}
