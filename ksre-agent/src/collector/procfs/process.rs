use super::proc_pid_io::ProcPidIO;
use super::proc_pid_stack::ProcPidStack;
use super::proc_pid_status::ProcPIDStatus;
use super::proc_pid_syscall::ProcPidSyscall;
use super::util;

#[derive(Default, Debug)]
pub struct Process {
    io: ProcPidIO,
    status: ProcPIDStatus,
    stack: ProcPidStack,
    syscall: ProcPidSyscall,
    tasks: Vec<Box<Process>>,
}

impl Process {
    pub fn read(pid_dir: String) -> Self {
        Process {
            io: ProcPidIO::read(&pid_dir),
            status: ProcPIDStatus::read(&pid_dir),
            stack: ProcPidStack::read(&pid_dir),
            syscall: ProcPidSyscall::read(&pid_dir),
            tasks: vec![],
        }
    }

    pub fn read_tasks(&mut self) {
        let pid = self.status.pid;
        let tids = util::read_dir_as_u64(format!("/proc/{pid}/task").as_str());
        for tid in tids {
            self.tasks
                .push(Box::new(Self::read(format!("/proc/{pid}/task/{tid}"))));
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let proc_stat = "2459 (containerd-shim) S 1 2459 580 0 -1 1077936128 107907 2489 0 0 3950 1842 0 1 20 0 12 0 27405 737468416 2279 18446744073709551615 65536 4382784 281474704322240 0 0 0 1002055680 0 2143420159 0 0 0 17 3 0 0 0 0 0 9109504 9295472 912633856 281474704326085 281474704326244 281474704326244 281474704326615 0";
    }
}
