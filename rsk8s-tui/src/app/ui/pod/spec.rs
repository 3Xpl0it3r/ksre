use ratatui::Frame;

use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::app::ui::util::outer_block;
use crate::kubernetes::api::{ContainerFields, PodFields};
pub fn draw_page_pod_spec(f: &mut Frame, state: &mut AppState, pod_fileds: Option<PodFields>, area: Rect) {
    let outer = outer_block(f, "pod status [l]: show log , [t]: exec", area);

}

/* fn container_widget(container: &PodFields)->Paragraph{

} */
