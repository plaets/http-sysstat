use serde::{Serialize};
use serde_json::{Value, json};
use systemstat::{System, Platform, data::CPULoad, data::DelayedMeasurement};
use chrono::{offset, Utc, Local, DateTime};
use std::io::{Result,Error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;

use crate::{DateFormat, IndexQuery};
use crate::interval_collector::{IntervalCollector, IntervalCollectorHandle};

fn print_err<T>(r: Result<T>) -> Option<T> {
    match r {
        Ok(val) => Some(val),
        Err(err) => {
            println!("error: {:?}", err);
            None
        }
    }
}

fn to_memorysize(cfg: &StatsConfig, size: u64) -> MemorySize {
    if cfg.human_readable {
        MemorySize::HumanReadable((size/(1024*1024)).to_string() + "MiB") 
    } else {
        MemorySize::Bytes(size)
    }
}

pub struct StatsConfig {
    pub date_format: DateFormat,
    pub human_readable: bool,
    pub query_other: HashMap<String, String>,
}

pub trait StatCollector: Send {
    fn new(/* program config */) -> Self where Self: Sized;
    fn collect(&self, config: &StatsConfig) -> serde_json::Result<Value>;
    fn name(&self) -> &'static str;
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Date {
    Epoch(u64),
    Local(String),
    Utc(String),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum MemorySize {
    Bytes(u64),
    HumanReadable(String),
}

struct TimeCollector;

impl StatCollector for TimeCollector {
    fn new() -> Self { Self{} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        
        serde_json::to_value(match cfg.date_format {
            DateFormat::Epoch => Date::Epoch(offset::Utc::now().timestamp() as u64),
            DateFormat::Local => Date::Local(offset::Local::now().to_string()),
            DateFormat::Utc => Date::Utc(offset::Utc::now().to_string()),
        })
    }

    fn name(&self) -> &'static str {
        "time"
    }
}

fn convert_to_date(cfg: &StatsConfig, date: DateTime<Utc>) -> Date {
   match cfg.date_format {
       DateFormat::Epoch => Date::Epoch(date.timestamp() as u64),
       DateFormat::Local => Date::Local(DateTime::<Local>::from(date).to_string()),
       DateFormat::Utc => Date::Utc(date.to_string()),
   }
}

struct UptimeCollector;

impl StatCollector for UptimeCollector {
    fn new() -> Self { Self {} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        
        Ok(json!({
            "uptime": print_err(sys.uptime()).map(|d| d.as_secs()),
            "boot_time": print_err(sys.boot_time()).map(|d| convert_to_date(cfg, d)),
        }))
    }

    fn name(&self) -> &'static str {
        "uptime"
    }
}

struct MemoryStatCollector; 

impl StatCollector for MemoryStatCollector {
    fn new() -> Self { Self {} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();

        serde_json::to_value(print_err(sys.memory()).map(|m| json!({
            "total": to_memorysize(cfg, m.total.as_u64()),
            "free": to_memorysize(cfg, m.free.as_u64()),
            "percentage_used": (((m.total.as_u64()-m.free.as_u64()) as f32)/(m.total.as_u64() as f32) * 100.0), 
            //todo: research if this is returns a valid, useful value
        })))
    }

    fn name(&self) -> &'static str {
        "mem_stats"
    }
}

#[derive(Serialize)]
pub struct CpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub interrupt: f32,
    pub idle: f32,
}

pub struct CpuLoadCollector {
    handle: IntervalCollectorHandle<CPULoad>
}

impl StatCollector for CpuLoadCollector {
    fn new() -> Self {
        let mut collector = IntervalCollector::new();
        collector
            .interval(Duration::from_millis(5000))
            .collect(|cpu: Arc<Mutex<Option<CPULoad>>>, measurement: Arc<Mutex<Option<Mutex<DelayedMeasurement<CPULoad>>>>>| {
                //yes, a double mutex
                //i want to die
                let sys = System::new();
                let measurement = measurement.clone();
                let mut measurement_guard = measurement.lock().unwrap(); //see if i care
                if let Some(measurement) = &*measurement_guard {
                    let measurement_guard = measurement.lock().unwrap();       
                    let cpu = cpu.clone();
                    let mut cpu_guard = cpu.lock().unwrap();
                    *cpu_guard = Some(measurement_guard.done().unwrap());
                }
                *measurement_guard = Some(Mutex::new(sys.cpu_load_aggregate().unwrap()));
            });

        Self {
            handle: collector.start()
        }
    }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let last_result = self.handle.last_result.clone();
        let last_result_lock = last_result.lock().unwrap();

        serde_json::to_value((*last_result_lock).as_ref().map(|c|
            json!({
                "user": c.user,
                "nice": c.nice,
                "system": c.system,
                "interrupt": c.interrupt,
                "idle": c.idle,
            })
        ))
    }

    fn name(&self) -> &'static str {
        "cpu_load"
    }
}

pub struct CpuLoadAvgCollector;

impl StatCollector for CpuLoadAvgCollector {
    fn new() -> Self { CpuLoadAvgCollector }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();

        serde_json::to_value(print_err(sys.load_average()).map(|l| json!({
            "one": l.one,
            "five": l.five,
            "fifteen": l.fifteen,
        })))
    }

    fn name(&self) -> &'static str {
        "cpu_avg"
    }
}

pub struct NetworkStatsCollector;

impl StatCollector for NetworkStatsCollector {
    fn new() -> NetworkStatsCollector { NetworkStatsCollector }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        serde_json::to_value(print_err(sys.networks()).map(|networks| 
            networks.values().map(|network| 
                json!({
                    "name": network.name.clone(),
                    "stats": print_err(sys.network_stats(&network.name)).map(|stats| json!({
                        "rx_bytes": to_memorysize(cfg, stats.rx_bytes.as_u64()),
                        "tx_bytes": to_memorysize(cfg, stats.tx_bytes.as_u64()),
                        "rx_packets": stats.rx_packets,
                        "tx_packets": stats.tx_packets,
                        "rx_errors": stats.rx_errors,
                        "tx_errors": stats.tx_errors,
                    }))
                })
            ).collect::<Vec<Value>>()
        ))
    }

    fn name(&self) -> &'static str {
        "net_stats"
    }
}

pub struct SocketStatCollector;

impl StatCollector for SocketStatCollector {
    fn new() -> SocketStatCollector { SocketStatCollector }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        serde_json::to_value(print_err(sys.socket_stats()).map(|stats|
            json!({
                "tcp_socks": stats.tcp_sockets_in_use as u64,
                "tcp_socks_orphaned": stats.tcp_sockets_orphaned as u64,
                "udp_socks": stats.udp_sockets_in_use as u64,
                "tcp6_socks": stats.tcp6_sockets_in_use as u64,
                "udp6_socks": stats.udp6_sockets_in_use as u64,
            })
        ))
    }

    fn name(&self) -> &'static str {
        "sock_stats"
    }
}

pub struct FilesystemStatsCollector;

impl StatCollector for FilesystemStatsCollector {
    fn new() -> FilesystemStatsCollector { FilesystemStatsCollector }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        serde_json::to_value(print_err(sys.mounts()).map(|mounts|
            mounts.iter().filter_map(|mount| 
                if mount.total.as_u64() != 0 {
                    Some(json!({
                        "name": mount.fs_mounted_from.clone(),
                        "type": mount.fs_type.clone(),
                        "free": to_memorysize(cfg, mount.free.as_u64()),
                        "avail": to_memorysize(cfg, mount.avail.as_u64()),
                        "total": to_memorysize(cfg, mount.total.as_u64()),
                    }))
                } else {
                    None
                }
            ).collect::<Vec<Value>>()
        ))
    }

    fn name(&self) -> &'static str {
        "fs_stats"
    }
}

pub fn get_all() -> Vec<Box<StatCollector>> {
    vec!(
        Box::new(TimeCollector::new()),
        Box::new(UptimeCollector::new()),
        Box::new(MemoryStatCollector::new()),
        Box::new(CpuLoadCollector::new()),
        Box::new(CpuLoadAvgCollector::new()),
        Box::new(SocketStatCollector::new()),
        Box::new(NetworkStatsCollector::new()),
        Box::new(FilesystemStatsCollector::new()),
    )
}
