use core::str;
use std::fs;

use ksre_lib::serializer::bytes::BytesCodec;
use ksre_lib_proc::ToBytes;

use super::Pid;

///
/// ProcIO contains IO statistics for each running process
/// rchar/wchar 统计的是用户态(存储层+pagecache)
/// syscr/syswr 统计的是内核态(存储层+pagecache)
/// read_bytes  统计的是从存储设备
/// https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n1584
#[derive(Default, Debug, ToBytes)]
pub(crate) struct IO {
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

impl IO {
    fn read(pid_dir: Pid) -> Self {
        let io_content = fs::read_to_string(format!("/proc/{pid_dir}/io")).unwrap();
        let mut proc = IO::default();
        proc.read_from(&io_content);
        proc
    }

    pub fn mock() -> Self {
        let mut proc = IO::default();

        let content = "rchar: 100\nwchar: 200\nsyscr: 300\nsyscw: 400\nread_bytes: 500\nwrite_bytes: 600\ncancelled_write_bytes: 700\n";
        proc.read_from(content);
        proc
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
}

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for IO {
    fn from(pid: Pid) -> Self {
        IO::read(pid)
    }
}

// BytesCodec[#TODO] (should add some comments)
impl BytesCodec for IO {
    fn byte_encode(&self) -> Vec<u8> {
        self.serialize()
    }

    fn byte_decode(&mut self, buffer: &[u8]) -> usize {
        self.deserialize(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_from() {
        let mut proc = IO::default();
        let content = "rchar: 100\nwchar: 200\nsyscr: 300\nsyscw: 400\nread_bytes: 500\nwrite_bytes: 600\ncancelled_write_bytes: 700\n";
        proc.read_from(content);

        assert_eq!(proc.rchar, 100);
        assert_eq!(proc.wchar, 200);
        assert_eq!(proc.syscr, 300);
        assert_eq!(proc.syscw, 400);
        assert_eq!(proc.read_bytes, 500);
        assert_eq!(proc.write_byte, 600);
        assert_eq!(proc.cancelled_write_bytes, 700);
    }

    #[test]
    fn test_serialize_deserialize() {
        let proc = IO {
            rchar: 100,
            wchar: 200,
            syscr: 300,
            syscw: 400,
            read_bytes: 500,
            write_byte: 600,
            cancelled_write_bytes: 700,
        };

        let serialized_data = proc.serialize();

        let mut deserialized_proc = IO::default();
        deserialized_proc.deserialize(&serialized_data);

        assert_eq!(deserialized_proc.rchar, 100);
        assert_eq!(deserialized_proc.wchar, 200);
        assert_eq!(deserialized_proc.syscr, 300);
        assert_eq!(deserialized_proc.syscw, 400);
        assert_eq!(deserialized_proc.read_bytes, 500);
        assert_eq!(deserialized_proc.write_byte, 600);
        assert_eq!(deserialized_proc.cancelled_write_bytes, 700);
    }
}
