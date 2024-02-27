use std::str::FromStr;

use chrono::{DateTime, Utc};
use ratatui::Frame;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, Paragraph};
use tracing::info;

use crate::app::AppState;
use crate::kubernetes::api::PodDescribe;

pub fn draw_pod_resource(
    f: &mut Frame,
    state: &AppState,
    pod_describe: Option<&PodDescribe>,
    area: Rect,
) {
    if pod_describe.is_none() {
        return;
    }
    /* let pod_describe = pod_describe.unwrap(); */
    /* let namespace: &str;
    let pod_name: &str;
    unsafe {
        namespace = &(*pod_describe.namespace);
        pod_name = &(*pod_describe.name);
    } */

    f.render_widget(state.metrics_buffer.widget(), area);
    /* if let Some(_namespaced_metrics) = state.pod_metrics_cache.get(namespace) {
        if let Some(_pod_metrics) = _namespaced_metrics.get(pod_name) {
        }
    } */

    /* f.render_widget(chart, area); */
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

fn generate_axis_labels(items: Vec<String>) -> Vec<Span<'static>> {
    items
        .into_iter()
        .map(|x| Span::styled(x, Style::default()))
        .collect::<Vec<Span<'static>>>()
}
