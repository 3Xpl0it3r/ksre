pub(crate) mod key;
pub(crate) use key::CusKey;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(CusKey),
    Error,
}

use crate::kubernetes::api::object::RtObject;
#[derive(Debug, Clone)]
pub enum KubeEvent<P: Clone, U: Clone> {
    OnAdd(RtObject<P, U>),
    OnDel(RtObject<P, U>),
}
