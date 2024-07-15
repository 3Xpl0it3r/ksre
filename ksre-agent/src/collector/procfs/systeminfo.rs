use core::f64;
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use ksre_lib::serializer::bytes::BytesCodec;

use super::process::ProcessState;
use super::util;

#[derive(Debug, Default)]
pub struct SystemInfo {
    pub timestamp: u64,
    pub processes: HashMap<u64, ProcessState>,
    load_avg: (f64, f64, f64),
}

impl SystemInfo {
    pub fn new_with_timestampl()-> Self {
        let mut systeminfo = SystemInfo::default();
        systeminfo.timestamp = current_time();
        systeminfo
    }
    pub fn open_fake_proc(&mut self) {
        for pid in 0..10 {
            let process_state = ProcessState::new(pid);
            self.processes.insert(pid, process_state);
        }
    }
    pub fn open_proc(&mut self) {
        for pid in util::read_dir_as_u64("/proc") {
            let process_state = ProcessState::new(pid);
            self.processes.insert(pid, process_state);
        }
    }

    fn read_loadavg(&mut self) {
        let conent = fs::read_to_string("/proc/loadavg").unwrap();
        let lines = conent.split(' ').collect::<Vec<&str>>();
        // loadavg data format
        self.load_avg.0 = lines[0].parse::<f64>().unwrap();
        self.load_avg.1 = lines[1].parse::<f64>().unwrap();
        self.load_avg.2 = lines[2].parse::<f64>().unwrap();
    }
}

// BytesCodec[#TODO] (should add some comments)
impl BytesCodec for SystemInfo {
    // encode
    fn byte_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(u64::to_le_bytes(self.timestamp));
        buffer.extend(u64::to_le_bytes(self.processes.len() as u64));
        for (pid, process) in self.processes.iter() {
            buffer.extend(u64::to_le_bytes(*pid));
            let proce_bytes = process.byte_encode();
            buffer.extend(u64::to_le_bytes(proce_bytes.len() as u64));
            buffer.extend(proce_bytes);
        }

        buffer
    }

    fn byte_decode(&mut self, buffer: &[u8]) -> usize {
        let mut offset = 0;
        self.timestamp = u64::from_le_bytes(buffer[offset..offset+8].try_into().unwrap());
        offset += 8;

        let item_num = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        for _ in 0..item_num {
            let pid = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let bytes_size =
                u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap()) as usize;
            offset += 8;
            let mut proc_state = ProcessState::new(pid);
            proc_state.byte_decode(&buffer[offset..offset + bytes_size]);
            offset += bytes_size;
            self.processes.insert(pid, proc_state);
        }
        offset
    }
}

#[inline]
fn current_time() -> u64 {
    let current = SystemTime::now();
    let duration_since_epoch = current.duration_since(UNIX_EPOCH).unwrap();
    duration_since_epoch.as_secs()
}
