use std::{fs, u64};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProcessState {
    R, // for running
    S, // for sleeping
    D, // for sleeping in an uninterruptible wait
    Z, // zombie
    T, // traced or stopped
    Unknown,
}

/// ProcPIDStatus represent Process status in human readble form
/// 这个里面包含的信息就是ps看到的信息, ps就是从这个里面获取数据,但是直接从/proc/<pid>/status
/// 可以获取到更多的信息
/// 这里只采集某一部分的我们觉得重要的数据
/// 从这里可以获取到更详细的文档https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n211
#[derive(Default, Debug)]
pub(crate) struct ProcPIDStatus {
    /// 当前进程可执行文件的名称
    pub name: String,
    pub state: u64,
    pub tgid: u64,
    /// 当前进程的pid
    pub pid: u64,
    /// 当前进程的父进程
    pub ppid: u64,
    /// 高峰虚拟内存, 单元kb, u64存储
    pub vm_peak: u64,
    pub vm_size: u64,
    pub vm_lck: u64,
    pub vm_hwm: u64,
    /// VMRSS = RssAnno + RssFile + RssShmen
    pub vm_rss: u64,
}

impl ProcPIDStatus {
    pub fn read(pid_dir: &str) -> Self {
        let conent = fs::read_to_string(format!("{pid_dir}/status")).unwrap();
        let mut status = ProcPIDStatus::default();

        status
    }

    #[inline]
    fn read_from(&mut self, content: &str) {
        for line in content.split('\n').collect::<Vec<&str>>() {
            let mut items = line.split(": ");
            let key = items.next().unwrap();
            match key {
                "Unmask" => {}
                "State" => {
                    // _S (sleep)_  => S (sleep) => (S, (sleep)) => S
                    let value = items.next().unwrap().trim().split(' ').next().unwrap();
                    match value {
                        "S" => self.state = 0,
                        "R" => self.state = 1,
                        "Z" => self.state = 2,
                        "D" => self.state = 3,
                        _ => self.state = 4,
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let test = r"Name:   containerd-shim
Umask:  0022
State:  S (sleeping)
Tgid:   2459
Ngid:   0
Pid:    2459
PPid:   1
TracerPid:      0
Uid:    0       0       0       0
Gid:    0       0       0       0
FDSize: 64
Groups: 0
NStgid: 2459
NSpid:  2459
NSpgid: 2459
NSsid:  580
VmPeak:   720184 kB
Mems_allowed:   1
Mems_allowed_list:      0
voluntary_ctxt_switches:        8
nonvoluntary_ctxt_switches:     9";
        let mut proc_pid_state = ProcPIDStatus::default();
        proc_pid_state.read_from(test);
        /* assert_eq!(proc_pid_state.state, ProcessState::S); */
    }
}
