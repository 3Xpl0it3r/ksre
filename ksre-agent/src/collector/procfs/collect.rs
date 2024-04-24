use core::f64;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use crate::collector::Collector;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use super::process::Process;
use super::util;

pub struct ProcCollector {
    tx_chan: Sender<Arc<ProcT>>,
    interval: u64, // 每隔period 执行一次
    task: JoinHandle<()>,
}

impl Collector for ProcCollector {
    type Item = Arc<ProcT>;
    fn chan(&self) -> Receiver<Self::Item> {
        self.tx_chan.subscribe()
    }

    fn run(&mut self, cancellation_token: tokio_util::sync::CancellationToken) {
        let writer = self.tx_chan.clone();
        let mut scrape_interval =
            tokio::time::interval(tokio::time::Duration::from_secs(self.interval));
        self.task = tokio::spawn(async move {
        })
    }
}

impl ProcCollector {
    pub fn new() -> ProcCollector {
        let (tx_sender, _) = broadcast::channel(1024);
        ProcCollector {
            interval: 5,
            tx_chan: tx_sender,
            task: tokio::spawn(async {}),
        }
    }
}

fn scrape() -> ProcT {
    let mut procfs = ProcT::new();
    procfs.open_proc();

    procfs
}

#[derive(Debug)]
pub struct ProcT {
    /* processes: Vec<Process>, */
    processes: HashMap<u64, Process>,
    load_avg: (f64, f64, f64),
}

impl ProcT {
    fn new() -> ProcT {
        ProcT {
            processes: HashMap::new(),
            load_avg: (0.0, 0.0, 0.0),
        }
    }
    fn open_proc(&mut self) {
        for pid in util::read_dir_as_u64("/proc") {
            let mut process = Process::read(format!("/proc/{pid}"));
            process.read_tasks();
            self.processes.insert(pid, process);
        }
    }

    fn read_loadavg(&mut self) {
        let conent = fs::read_to_string("/proc/loadavg").unwrap();
        let lines = conent.split(' ').collect::<Vec<&str>>();
        // loadavg data format
        // 0.36 0.23 0.31 1/970 394071
        // 前三个字段分别代表1分钟,5分钟,15分钟的cpu负载情况
        // 第四个字段是当前处于Running的进程数/总进程数
        self.load_avg.0 = lines[0].parse::<f64>().unwrap();
        self.load_avg.1 = lines[1].parse::<f64>().unwrap();
        self.load_avg.2 = lines[2].parse::<f64>().unwrap();
    }
}
