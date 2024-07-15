mod collector;
pub mod process;

pub(crate) mod meminfo;
pub(crate) mod symbols;
pub mod systeminfo;

mod util;

pub(crate) enum ObjectKind {
    MemInfo,
    PidIO,
    PidMaps,
    PidSamps,
    PidStack,
    PidStat,
    PidStatus,
    PidSyscall,
}

pub(super) const MAGIC_SPLIT_NUMBER: u64 = 0xFFBAABFF;
pub(super) const MAGIC_ENDING_NUMBER: u64 = 0xFFABBAFF;

pub use collector::Collector;
