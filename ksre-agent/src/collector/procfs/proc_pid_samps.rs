use std::fs;

use tokio_util::bytes::buf;

use crate::bytes;

/// ProcPidSamps 作为maps的一个扩展, 用来展示每个段的内存消耗情况, 它是/proc/pid/maps,
/// 所以我们这里只保存smaps就足够了,它信息更为详细
/// 一个更为详细的展示
/// ref: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n418
///
/// 第一列和maps是一样的 格式如下:
/// address  permis offset dev inode pathname
/// 相关计算公式如下:
/// all_rss = [smap.rss for smap in smaps]
/// all_pss = [smap.pss for smap in smaps]
/// all_uss = [smap.private_clean + smap.private_dirty for smap in smaps]
#[derive(Default)]
pub struct ProcPidSmaps {
    smaps: Vec<ProcPidSmap>,
}

impl ProcPidSmaps {
    pub fn read(pid_dir: &str) -> Self {
        let conent =
            fs::read_to_string(format!("{pid_dir}/smaps")).expect("smaps file is not found");
        let mut smaps = ProcPidSmaps::default();
        smaps.read_from(&conent);
        smaps
    }

    #[inline]
    fn read_from(&mut self, conent: &str) {
        let lines = conent.split('\n').collect::<Vec<&str>>();
        let mut smap = ProcPidSmap::default();
        for line in lines {
            if line.starts_with("VmFlags:") {
                let tmp = smap;
                smap = ProcPidSmap::default();
                self.smaps.push(tmp);
                continue;
            }
            let token = line.trim().split(' ').collect::<Vec<&str>>();
            if token.len() <= 2 {
                continue;
            }
            match token[0].trim_end_matches(':') {
                "Size" => {
                    smap.size = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file");
                }
                "Rss" => {
                    smap.rss = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Pss" => {
                    smap.pss = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Shared_Clean" => {
                    smap.shared_clean = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Shared_Dirty" => {
                    smap.shared_dirty = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Private_Clean" => {
                    smap.private_clean = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Private_Dirty" => {
                    smap.private_dirty = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Referenced" => {
                    smap.referenced = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Anonymous" => {
                    smap.anonymous = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "AnonHugePages" => {
                    smap.anonhugepages = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Swap" => {
                    smap.swap = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "KernelPageSize" => {
                    smap.kernel_page_size = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "MMUPageSize" => {
                    smap.mmu_page_size = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }
                "Locked" => {
                    smap.locked = token[token.len() - 2]
                        .parse::<u64>()
                        .expect("invalid smap file")
                }

                _ => {}
            }
        }
    }
}

/// 相关计算公式如下:
/// rss=(shared_clean_kb+shared_dirty_kb+private_celan_kb + private_dirty_kb)
/// uss=(private_clean_kb + private_ditry_kb)
#[derive(Default)]
pub struct ProcPidSmap {
    /// first line information
    address_range: String,
    perm: String,
    offset: String,
    dev: String,
    inode: String,
    path_name: String,
    /// 剩下的字段
    /// 当前段的内存大小
    size: u64,
    /// 当前段常驻内存大小
    rss: u64,
    /// pss(proportional share size) 实际使用的内存(但是共享内存部分按照比例计算,share的部分/share进程个数)
    pss: u64,

    /// 需要注意的时候share-able page也会被计算到private_[clean/dirty],
    /// 只有当它们被真正的被share的时候才会被计算到shared_[clean|dirty]_kb里面
    shared_clean: u64,
    shared_dirty: u64,
    private_clean: u64,
    private_dirty: u64,
    referenced: u64,

    anonymous: u64,
    anonhugepages: u64,

    swap: u64,
    kernel_page_size: u64,
    mmu_page_size: u64,
    locked: u64,
    vm_flags: Vec<String>,
}

// ProcPidSmap[#TODO] (should add some comments)
impl ProcPidSmap {
    fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // encode address range
        let mut bytes = self.address_range.as_bytes();
        buffer.extend(bytes::Varint::encode_u64(bytes.len() as u64));
        let mut tmp_buf: Vec<u8> = bytes.into();
        buffer.extend(tmp_buf);

        // encode perm
        bytes = self.perm.as_bytes();
        buffer.extend(bytes::Varint::encode_u64(bytes.len() as u64));
        buffer.extend(Into::<Vec<u8>>::into(bytes));

        // encode offset
        bytes = self.offset.as_bytes();
        buffer.extend(bytes::Varint::encode_u64(bytes.len() as u64));
        buffer.extend(Into::<Vec<u8>>::into(bytes));

        // encode dev
        bytes = self.dev.as_bytes();
        buffer.extend(bytes::Varint::encode_u64(bytes.len() as u64));
        tmp_buf = bytes.into();
        buffer.extend(tmp_buf);

        // encode idnode
        bytes = self.inode.as_bytes();
        buffer.extend(bytes::Varint::encode_u64(bytes.len() as u64));
        tmp_buf = bytes.into();
        buffer.extend(tmp_buf);

        // encode size
        buffer.extend(bytes::Varint::encode_u64(self.size));
        // encode rss
        buffer.extend(bytes::Varint::encode_u64(self.rss));
        // encode pss
        buffer.extend(bytes::Varint::encode_u64(self.pss));
        // encode shared_clean
        buffer.extend(bytes::Varint::encode_u64(self.shared_clean));
        buffer.extend(bytes::Varint::encode_u64(self.shared_dirty));

        buffer.extend(bytes::Varint::encode_u64(self.private_clean));
        buffer.extend(bytes::Varint::encode_u64(self.private_dirty));

        buffer.extend(bytes::Varint::encode_u64(self.referenced));

        buffer.extend(bytes::Varint::encode_u64(self.anonymous));

        buffer.extend(bytes::Varint::encode_u64(self.anonhugepages));

        buffer.extend(bytes::Varint::encode_u64(self.swap));

        buffer.extend(bytes::Varint::encode_u64(self.kernel_page_size));

        buffer.extend(bytes::Varint::encode_u64(self.mmu_page_size));

        buffer.extend(bytes::Varint::encode_u64(self.locked));

        // encode flage number
        let flags_number = self.vm_flags.len();
        for _ in 0..flags_number {
            // encode string len
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let test_file = "./src/collector/procfs/test/smaps";
        let content = fs::read_to_string(test_file).unwrap();
        let mut proc_smaps = ProcPidSmaps::default();
        proc_smaps.read_from(&content);
        assert_eq!(9, proc_smaps.smaps.len());
        assert_eq!(17168, proc_smaps.smaps[0].size);
        assert_eq!(6212, proc_smaps.smaps[0].referenced);
        assert_eq!(8332, proc_smaps.smaps[1].pss);
    }
}
