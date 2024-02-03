use crate::kubernetes::api::RtObject;
use crossterm::event::KeyEvent;


mod key;

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Error,
}

#[derive(Debug, Clone)]
pub enum KubeEvent<P: Clone, U: Clone> {
    OnAdd(RtObject<P, U>),
    OnDel(RtObject<P, U>),
}
