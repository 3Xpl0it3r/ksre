mod index;
mod list;
mod status;
mod spec;
mod tty;


pub(super) use status::draw_page_pod_status;
pub(super) use tty::draw_page_pod_tty;
pub(super) use spec::draw_page_pod_spec;
pub(super) use list::draw_page_pod_list;


pub use index::draw_page_index;


