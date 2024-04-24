use std::fs;
use std::path::PathBuf;

pub(super) fn read_dir(path: &str) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![];
    for entry in fs::read_dir(path).unwrap() {
        if entry.is_err() {
            continue;
        }
        let file_item = entry.unwrap();
        if file_item.path().is_dir() {
            dirs.push(file_item.path());
        }
    }
    dirs
}

pub(super) fn read_dir_as_u64(path: &str) -> Vec<u64> {
    let mut pids = vec![];
    for file_item in fs::read_dir(path).unwrap() {
        if file_item.is_err() {
            continue;
        }
        let file_item = file_item.unwrap();
        if file_item.path().is_dir() {
            if let Ok(pid) = file_item.path().to_str().unwrap().trim().parse::<u64>() {
                pids.push(pid);
            }
        }
    }
    pids
}
