use std::fs;

/// ProcMemInfo 主要展示内存分布情况以及内存使用情况
/// reference: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/filesystems/proc.txt?h=v5.1-rc6#n859
#[derive(Default)]
struct ProcMemInfo {
    mem_total: u32,
    mem_free: u32,
    mem_available: u32,
    buffer: u32,
    cached: u32,
    swap_cached: u32,
    active: u32,
    inactive: u32,
    active_anon: u32,
    inactive_anon: u32,
    active_file: u32,
    inactive_file: u32,
    unevictable: u32,
    mlocked: u32,
    swap_total: u32,
    swap_free: u32,
    dirty: u32,
    write_back: u32,
    anon_pages: u32,
    mapped: u32,
    shmem: u32,
    slab: u32,

    s_reclaimable: u32,
    s_unreclaim: u32,
    kernel_stack: u32,
    page_tables: u32,
    nfs_unstable: u32,
    bounce: u32,
    writeback_tmp: u32,
    ///  ([total RAM pages] - [total huge TLB pages]) * overcommit_ratio
    ///  ───────────────────────────────────────────────────────────────── + [total swap pages]
    commit_limit: u32,
    /// committed allocations  已经提交了的allocation
    committed_as: u32,

    vmalloc_total: u32,
    vmalloc_used: u32,
    percpu: u32,
    hardware_corrupted: u32,
    annon_huge_pages: u32,
    cma_total: u32,
    cma_free: u32,

    huge_pages_total: u32,
    huge_pages_free: u32,
    huge_pages_rsvd: u32,
    hug_pages_surp: u32,
    hugepagesize: u32,

    direct_map4k: u32,
    direct_map2m: u32,
    direct_map1g: u32,
}

// ProcMemInfo[#TODO] (should add some comments)
impl ProcMemInfo {
    fn read() -> Self {
        let content = fs::read_to_string("/proc/meminfo").expect("/proc/meminfo is not existed");

        let mut meminfo = ProcMemInfo::default();

        let lines = content.split('\n').collect::<Vec<&str>>();
        for line in lines {
            let mut token = line.trim().split(' ').collect::<Vec<&str>>();
            if token.len() > 2 {
                token.remove(token.len() - 1);
            }
            match token[0] {
                "MemTotal" => {
                    meminfo.mem_total = token[1].parse::<u32>().expect("invalid meminfo conent")
                }
                "MemFree" => meminfo.mem_free = token[1].parse::<u32>().expect("invalid meminfo "),
                _ => {}
            }
        }
        meminfo
    }
}
