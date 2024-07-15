use core::panic;
use std::fs;

use super::Pid;

/// /proc/<pid>/syscall 文件显示进程最近系统调用的信息, 有三种形式
/// 61 0xffffffffffffffff 0x7ffd20bafaec 0x0 0x0 0x0 0x0 0x7ffd20bafbe0 0x7f394f823407
/// 第一个字段代表系统调用的编号 >0 代表阻塞在系统调用处, -1 代表阻塞在用户态 -2代表没有阻塞
#[derive(Default, Debug)]
pub struct Syscall {
    /// 第一个字段代表系统调用id
    syscall_id: i32,
    /// arg0-args5 为系统调用参数, 下面参数ignored
    _arg0: String,
    _arg1: String,
    _arg2: String,
    _arg3: String,
    _arg4: String,
    _arg5: String,
    /// 栈指针
    _esp: Option<i8>,
    _eip: Option<i8>,
}

impl Syscall {
    fn read(pid: Pid) -> Self {
        let mut proc_syscall = Self::default();
        let content = fs::read_to_string(format!("/proc/{pid}/syscall"))
            .expect("/proc/<pid>/syscall open failed");
        proc_syscall.read_from(&content);
        proc_syscall
    }

    #[inline]
    fn read_from(&mut self, content: &str) {
        let token = content.split(' ').collect::<Vec<&str>>();
        if let Ok(syscall_id) = token[0].parse::<i32>() {
            self.syscall_id = syscall_id;
        } else if token[0].eq("running") {
            self.syscall_id = -2;
        } else {
            panic!("unexped syscall content");
        }
    }
}

// From<Pid[#TODO] (should add some comments)
impl From<Pid> for Syscall {
    fn from(pid: Pid) -> Self {
        Syscall::read(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let content =
            "61 0xffffffffffffffff 0x7ffd20bafaec 0x0 0x0 0x0 0x0 0x7ffd20bafbe0 0x7f394f823407";
        let mut pid_syscall = Syscall::default();
        pid_syscall.read_from(content);
        assert_eq!(61, pid_syscall.syscall_id);
    }
}
