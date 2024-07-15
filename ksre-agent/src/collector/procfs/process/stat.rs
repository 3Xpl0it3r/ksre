use super::Pid;

use super::constant::PState;

/// /proc/<pid>/stat contains status information about a process. This information is used by `ps` command
#[derive(Debug)]
pub struct Stat {
    pid: u64,
    comm: String,
    state: PState,
    // The pid of the parent of this process
    ppid: u64,
    // The process group ID of the process
    pgrp: u64,
    // The session ID of the process
    session: u64,
    // The controlling terminal of the process
    tty_nr: u64,
    // The Id of the foreground process group of the controlling terminal of the process
    tpgid: u64,
    // The kernel flags word of the process. For bit meaning, see the PF_* defines
    flags: u64,
    // The number of minor faults that the process has made which have not required loading a
    // memory page from disk
    minflt: u64,
    // The number of minor faults that the prcoess's waited-for children have made
    cminflt: u64,
    // The number of major faults the process has made which have required loading a memory page
    // from disk
    majflt: u64,
    // The number of major fault that the process's waited-for children have made
    cmajflt: u64,
    // Amount of time that this process has been scheduled in user mode. This includes guest
    // time(guest time time spent running a virtual cpu)
    utime: u64,
    // Amount of time that this process has been schduled in kernel mode, measured in click ticks
    stime: u64,
    // Amount of time that this process's waited-for children have been scheduled in kernel mode,
    // measured in click tickjs
    cutime: u64,
    cstime: u64,
}

// Stat[#TODO] (should add some comments)
impl Stat {
    fn read(_pid: Pid) -> Self {
        unreachable!("impl this")
    }
}

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for Stat {
    fn from(pid: Pid) -> Self {
        Stat::read(pid)
    }
}
