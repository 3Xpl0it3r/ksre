pub mod app;
pub mod event;
mod kubernetes;
pub mod tui;

pub use kubernetes::{helper::default_kubernetes_client, reflector::PodReflector};
