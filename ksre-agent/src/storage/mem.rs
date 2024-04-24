use std::sync::Arc;

use crate::collector::procfs::collect::ProcT;

// MemStorage[#TODO] (shoule add some comments )
#[derive(Default)]
struct MemStorage {
    proc_metrics: Vec<Arc<ProcT>>,
}

// MemStorage[#TODO] (should add some comments)
impl MemStorage {
    fn add() -> Self {
        unreachable!("impl this")
    }
}
