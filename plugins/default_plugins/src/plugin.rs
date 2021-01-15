use systemstat::{System, Platform, data::CPULoad, data::DelayedMeasurement};
use chrono::{offset};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http_sysstat_pluginlib::serde_json;
use http_sysstat_pluginlib::serde_json::{Value, json};
use http_sysstat_pluginlib::interval_collector::{IntervalCollector, IntervalCollectorHandle};
use http_sysstat_pluginlib::stats_collector::{StatsCollector, DateFormat, StatsConfig, ConfigValue};
use http_sysstat_pluginlib::utils::*;

pub struct TimeCollector;
impl StatsCollector for TimeCollector {
    fn new() -> Self { Self{} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
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

pub struct UptimeCollector;
impl StatsCollector for UptimeCollector {
    fn new() -> Self { Self {} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();
        
        Ok(json!({
            "uptime": print_err(sys.uptime()).map(|d| d.as_secs()), //TODO: make this human readable
            "boot_time": print_err(sys.boot_time()).map(|d| convert_to_date(cfg, d)),
        }))
    }

    fn name(&self) -> &'static str {
        "uptime"
    }
}

pub struct MemoryStatsCollector; 
impl StatsCollector for MemoryStatsCollector {
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

pub struct CpuLoadCollector {
    handle: IntervalCollectorHandle<CPULoad>
}

impl StatsCollector for CpuLoadCollector {
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
impl StatsCollector for CpuLoadAvgCollector {
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
impl StatsCollector for NetworkStatsCollector {
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

pub struct SocketStatsCollector;
impl StatsCollector for SocketStatsCollector {
    fn new() -> SocketStatsCollector { SocketStatsCollector }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        let sys = System::new();

        let cfg = cfg.plugin_config.clone();

        if let Some(ConfigValue::Map(plugin_cfg)) = &cfg.get(&String::from(self.name())) {
            if let Some(ConfigValue::Bool(disabled)) = &plugin_cfg.get("disabled") {
                if *disabled {
                    return Ok(json!({}))
                }
            }
        }

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
impl StatsCollector for FilesystemStatsCollector {
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

pub fn get_all() -> Vec<Box<dyn StatsCollector>> {
    vec!(
        Box::new(TimeCollector::new()),
        Box::new(UptimeCollector::new()),
        Box::new(MemoryStatsCollector::new()),
        Box::new(CpuLoadCollector::new()),
        Box::new(CpuLoadAvgCollector::new()),
        Box::new(SocketStatsCollector::new()),
        Box::new(NetworkStatsCollector::new()),
        Box::new(FilesystemStatsCollector::new()),
    )
}
