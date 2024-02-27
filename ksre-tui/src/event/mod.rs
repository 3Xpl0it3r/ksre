use crate::kubernetes::api::RtObject;


mod key;

pub use key::CusKey;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(CusKey),
    Error,
}


#[derive(Debug, Clone)]
pub enum KubeEvent<P: Clone, U: Clone> {
    OnAdd(RtObject<P, U>),
    OnDel(RtObject<P, U>),
}
