pub(crate) mod collect;
pub(crate) mod serial;

pub(crate) mod proc_meminfo;
pub(crate) mod proc_pid_io;
pub(crate) mod proc_pid_maps;
pub(crate) mod proc_pid_samps;
pub(crate) mod proc_pid_stack;
pub(crate) mod proc_pid_stat;
pub(crate) mod proc_pid_status;
pub(crate) mod proc_pid_syscall;
pub(crate) mod process;

pub(crate) mod symbols;
pub(super) mod util;

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

const MAGIC_NUMBER: u64 = 0xFFABCDEF;
