use systemstat::{System, Platform, data::CPULoad};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::io::Result;

pub struct CpuLoadCollector {
    pub ms_interval: u64,
    pub last_result: Arc<RwLock<Option<CPULoad>>>,
    sys: System,
}

fn print_err<T>(r: Result<T>) -> Option<T> {
    match r {
        Ok(val) => Some(val),
        Err(err) => {
            println!("error: {:?}", err);
            None
        }
    }
}

pub struct CollectorHandle {
    pub last_result: Arc<RwLock<Option<CPULoad>>>,
    pub join_handle: thread::JoinHandle<()>,
}

pub fn spawn_collector_thread(ms_interval: u64) -> CollectorHandle {
    let collector = CpuLoadCollector::new(ms_interval);
    CollectorHandle {
        last_result: collector.last_result.clone(),
        join_handle: thread::spawn(move || collector.start())
    }
}

impl CpuLoadCollector {
    pub fn new(ms_interval: u64) -> CpuLoadCollector {
        CpuLoadCollector {
            ms_interval,
            last_result: Arc::new(RwLock::new(None)),
            sys: System::new(),
        }
    }

    pub fn start(&self) {
        loop {
            let cpu_load = self.sys.cpu_load_aggregate();
            thread::sleep(Duration::from_millis(self.ms_interval));
            let mut last_res_guard = (*self.last_result).write().unwrap();
            (*last_res_guard) = print_err(cpu_load.map(|c| c.done().unwrap()));
        }
    }
}

pub fn test() {
    let handle = spawn_collector_thread(5000);
}
