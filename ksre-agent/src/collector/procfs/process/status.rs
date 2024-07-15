use std::{fs, u64};

use ksre_lib::serializer::bytes::BytesCodec;
use ksre_lib_proc::ToBytes;

use super::Pid;

/// ProcPIDStatus represent Process status in human readble form
/// 这个里面包含的信息就是ps看到的信息, ps就是从这个里面获取数据,但是直接从/proc/<pid>/status
/// 可以获取到更多的信息
/// 这里只采集某一部分的我们觉得重要的数据
/// 从这里可以获取到更详细的文档https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n211
#[derive(Default, Debug, ToBytes)]
pub(crate) struct Status {
    /// 当前进程可执行文件的名称
    pub name: String,
    pub umask: String,
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
    pub vm_pin: u64,
    pub vm_hwm: u64,
    /// VMRSS = RssAnno + RssFile + RssShmen
    pub vm_rss: u64,
    pub rss_anon: u64,
    pub rss_file: u64,
    pub rss_shmem: u64,
}

impl Status {
    fn read(pid: Pid) -> Self {
        let conent = fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
        let mut status = Status::default();
        status.read_from(&conent);
        status
    }

    pub fn mock() -> Self {
        let mut status = Status::default();
        let content = "Name: test_process\nUmask: 0022\nState: R (running)\nTgid: 1234\nPid: 5678\nPPid: 9876\nVmPeak: 1024 kb\nVmSize: 512 kb\nVmLck: 0 kb\nVmPin: 256 kb\nVmHWM: 2048 kb\nVmRSS: 1024 kb\nRssAnon: 512 kb\nRssFile: 256 kb\nRssShmem: 128 kb";
        status.read_from(content);
        status
    }

    #[inline]
    fn read_from(&mut self, content: &str) {
        for line in content.split('\n').collect::<Vec<&str>>() {
            let mut items = line.split(": ");
            let key = items.next().unwrap();
            match key {
                "Name" => self.name = items.next().unwrap().trim().into(),
                "Umask" => self.umask = items.next().unwrap().trim().into(),
                "State" => {
                    let state = items.next().unwrap();
                    self.state = if state.contains('R') {
                        0
                    } else if state.contains('S') {
                        1
                    } else if state.contains('D') {
                        2
                    } else if state.contains('Z') {
                        3
                    } else if state.contains('T') {
                        4
                    } else {
                        5
                    };
                }
                "Tgid" => {
                    let tgid = items.next().unwrap().trim();
                    self.tgid = tgid.parse::<u64>().unwrap();
                }
                "Pid" => {
                    let pid = items.next().unwrap().trim();
                    self.pid = pid.parse::<u64>().unwrap();
                }
                "PPid" => {
                    let ppid = items.next().unwrap().trim();
                    self.ppid = ppid.parse::<u64>().unwrap();
                }
                "VmPeak" => self.vm_peak = parse_vm(items.next().unwrap()),
                "VmSize" => self.vm_size = parse_vm(items.next().unwrap()),
                "VmLck" => self.vm_lck = parse_vm(items.next().unwrap()),
                "VmPin" => self.vm_pin = parse_vm(items.next().unwrap()),
                "VmHWM" => self.vm_hwm = parse_vm(items.next().unwrap()),
                "VmRSS" => self.vm_rss = parse_vm(items.next().unwrap()),
                "RssAnon" => self.rss_anon = parse_vm(items.next().unwrap()),
                "RssFile" => self.rss_file = parse_vm(items.next().unwrap()),
                "RssShmem" => self.rss_shmem = parse_vm(items.next().unwrap()),
                _ => {}
            }
        }
    }
}

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for Status {
    fn from(pid: Pid) -> Self {
        Status::read(pid)
    }
}

// BytesCodec[#TODO] (should add some comments)
impl BytesCodec for Status {
    fn byte_encode(&self) -> Vec<u8> {
        self.serialize()
    }

    fn byte_decode(&mut self, buffer: &[u8]) -> usize {
        self.deserialize(buffer)
    }
}

#[inline]
fn parse_vm(value: &str) -> u64 {
    let value = value.trim().trim_end_matches("kb").trim();
    value.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vm() {
        assert_eq!(parse_vm("1234kb"), 1234);
        assert_eq!(parse_vm("5678 kb"), 5678);
        assert_eq!(parse_vm("9999 kb   "), 9999);
    }

    #[test]
    fn test_status_read_from() {
        let mut status = Status::default();
        let content = "Name: test_process\nUmask: 0022\nState: R (running)\nTgid: 1234\nPid: 5678\nPPid: 9876\nVmPeak: 1024 kb\nVmSize: 512 kb\nVmLck: 0 kb\nVmPin: 256 kb\nVmHWM: 2048 kb\nVmRSS: 1024 kb\nRssAnon: 512 kb\nRssFile: 256 kb\nRssShmem: 128 kb";
        status.read_from(content);

        assert_eq!(status.name, "test_process");
        assert_eq!(status.umask, "0022");
        assert_eq!(status.state, 0); // State field is not parsed in the provided code
        assert_eq!(status.tgid, 1234);
        assert_eq!(status.pid, 5678);
        assert_eq!(status.ppid, 9876);
        assert_eq!(status.vm_peak, 1024);
        assert_eq!(status.vm_size, 512);
        assert_eq!(status.vm_lck, 0);
        assert_eq!(status.vm_pin, 256);
        assert_eq!(status.vm_hwm, 2048);
        assert_eq!(status.vm_rss, 1024);
        assert_eq!(status.rss_anon, 512);
        assert_eq!(status.rss_file, 256);
        assert_eq!(status.rss_shmem, 128);
    }
}
