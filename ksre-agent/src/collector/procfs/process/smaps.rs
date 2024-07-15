use std::fs;

use super::Pid;
use ksre_lib::serializer::bytes::BytesCodec;
use ksre_lib_proc::ToBytes;

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
#[derive(Default, Debug)]
pub struct Smaps {
    smaps: Vec<TaskSmap>,
}

impl Smaps {
    fn read(pid: Pid) -> Self {
        let conent =
            fs::read_to_string(format!("/proc/{pid}/smaps")).expect("smaps file is not found");
        let mut smaps = Smaps::default();
        smaps.read_from(&conent);
        smaps
    }

    #[inline]
    fn read_from(&mut self, conent: &str) {
        let lines = conent.split('\n').collect::<Vec<&str>>();
        let mut smap = TaskSmap::default();
        for line in lines {
            if line.starts_with("VmFlags:") {
                let tmp = smap;
                smap = TaskSmap::default();
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

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for Smaps {
    fn from(pid: Pid) -> Self {
        Smaps::read(pid)
    }
}

/// 相关计算公式如下:
/// rss=(shared_clean_kb+shared_dirty_kb+private_celan_kb + private_dirty_kb)
/// uss=(private_clean_kb + private_ditry_kb)
#[derive(Default, Debug, ToBytes)]
pub struct TaskSmap {
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
    /* vm_flags: Vec<String>, */
}
