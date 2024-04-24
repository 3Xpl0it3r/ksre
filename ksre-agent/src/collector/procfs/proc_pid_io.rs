use core::str;
use std::fs;

use tokio_util::bytes::buf;

use crate::bytes;

use super::{ObjectKind, MAGIC_NUMBER};

///
/// ProcIO contains IO statistics for each running process
/// rchar/wchar 统计的是用户态(存储层+pagecache)
/// syscr/syswr 统计的是内核态(存储层+pagecache)
/// read_bytes  统计的是从存储设备
/// https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n1584
#[derive(Default, Debug)]
pub(crate) struct ProcPidIO {
    /// 统计从磁盘读取了多少字节,这个仅仅统计由read()和pread()函数读取的数据总和;
    /// 统计的对象包括tty IO
    /// 它统计的对象不受是否真的需要从物理磁盘读取数据的影响,它有可能是从pagecache里面读取到的)
    rchar: u64,
    /// 和rchar类似
    wchar: u64,
    /// 由syscall 触发的IO读操作的统计(包含但不局限于read()和preaad())
    /// 和rchar类似 rchar是用户态
    /// 这个地方统计是内核态
    syscr: u64,
    /// 由syscall 触发的IO读操作的统计(包含但不局限于read()和preaad())
    syscw: u64,
    /// 统计当前进程往存储设备上读入字节总数统计
    read_bytes: u64,
    /// 统计当前进程往存储设备上写入字节总数统计
    write_byte: u64,
    /// 用来纠错
    cancelled_write_bytes: u64,
}

const PROC_IOPATH: &str = "io";

impl ProcPidIO {
    pub fn read(pid_dir: &str) -> Self {
        let io_content = fs::read_to_string(format!("{pid_dir}/io")).unwrap();
        let mut proc = ProcPidIO::default();
        proc.read_from(&io_content);
        proc
    }
    // raw data format is:
    // |type | keysize | datalen  |
    // | 1B  |  1B     |   n * 8B |

    fn deccode(content: &str) -> Self {
        ProcPidIO::default()
    }

    #[inline]
    fn read_from(&mut self, content: &str) {
        let lines = content.split('\n').collect::<Vec<&str>>();
        for line in lines {
            let mut item = line.split(": ");
            let key = item.next().unwrap_or_default();
            let value = item
                .next()
                .map(|x| x.parse::<u64>().unwrap_or_default())
                .unwrap_or_default();
            match key {
                "rchar" => self.rchar = value,
                "wchar" => self.wchar = value,
                "syscr" => self.syscr = value,
                "syscw" => self.syscw = value,
                "read_bytes" => self.read_bytes = value,
                "write_bytes" => self.write_byte = value,
                "cancelled_write_bytes" => self.cancelled_write_bytes = value,
                _ => {}
            }
        }
    }

    /// layout
    /// |OBJ_TYPE | data  | END_MAGIC_NUMBER |
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(vec![ObjectKind::PidIO as u8]);
        buffer.extend(bytes::Varint::encode_u64(self.rchar));
        buffer.extend(bytes::Varint::encode_u64(self.wchar));
        buffer.extend(bytes::Varint::encode_u64(self.syscr));
        buffer.extend(bytes::Varint::encode_u64(self.syscw));
        buffer.extend(bytes::Varint::encode_u64(self.read_bytes));
        buffer.extend(bytes::Varint::encode_u64(self.cancelled_write_bytes));
        buffer.extend(bytes::Varint::encode_u64(MAGIC_NUMBER));

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let testcases = r"rchar: 323934931
wchar: 323929600
syscr: 632687
syscw: 632675
read_bytes: 0
write_bytes: 323932160
cancelled_write_bytes: 0
";
        let mut proc_io = ProcPidIO::default();
        proc_io.read_from(testcases);

        assert_eq!(323934931, proc_io.rchar);
        assert_eq!(323929600, proc_io.wchar);
        assert_eq!(632687, proc_io.syscr);
        assert_eq!(632675, proc_io.syscw);
        assert_eq!(0, proc_io.read_bytes);
        assert_eq!(323932160, proc_io.write_byte);
        assert_eq!(0, proc_io.cancelled_write_bytes);
    }
}
