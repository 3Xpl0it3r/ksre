use ratatui::Frame;

use ratatui::layout::Rect;

use crate::app::state::AppState;
use crate::{app::ui::util::outer_block, kubernetes::api::pod::PodDescribe};

pub fn draw_page_pod_spec(
    f: &mut Frame,
    _state: &mut AppState,
    _pod_fileds: PodDescribe,
    area: Rect,
) {
    let _outer = outer_block(f, "pod status [l]: show log , [t]: exec", area);
}

/* fn container_widget(container: &PodFields)->Paragraph{

} */
