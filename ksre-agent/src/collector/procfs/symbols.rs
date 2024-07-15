use std::collections::HashMap;
use std::fs;

use lazy_static::lazy_static;

#[cfg(platform = "x86_64")]
const UNISTD_HEAD_FILE: &str = "/usr/include/asm-generic/unistd.h";

#[cfg(platform = "arm")]
const UNISTD_HEAD_FILE: &str = "/usr/include/asm-generic/unistd.h";

lazy_static! {
    pub static ref SYMBOLICS: HashMap<i32, String> = get_symbols(UNISTD_HEAD_FILE);
}

fn get_symbols(unistd_file: &str) -> HashMap<i32, String> {
    let mut syscall_define = HashMap::<i32, String>::from([
        (-1, "Not Syscall".to_string()),
        (-2, "Not Block".to_string()),
    ]);
    let content = fs::read_to_string(unistd_file).unwrap();
    let lines = content.split('\n').collect::<Vec<&str>>();

    for line in lines {
        if !line.starts_with("#define __NR") {
            continue;
        }
        let token = line.split(' ').collect::<Vec<&str>>();
        if let Ok(key) = token[token.len() - 1].parse::<i32>() {
            syscall_define.insert(key, token[1].to_string());
        }
    }
    syscall_define
}

pub enum ProcessSymbol {
    ProcMemInfo,
    ProcPidIo,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let unistd_hf = "./src/collector/procfs/test/unistd.h";
        let syscall_map = get_symbols(unistd_hf);

        assert_eq!("__NR_name_to_handle_at", syscall_map.get(&264).unwrap());
        assert_eq!("__NR_nanosleep", syscall_map.get(&101).unwrap());
        assert_eq!("__NR_sendmmsg", syscall_map.get(&269).unwrap());
    }
}
