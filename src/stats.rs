use serde::{Serialize};
use systemstat::{System, Platform};
use chrono::{offset, Utc, Local, DateTime};
use std::io::Result;

use crate::{DateFormat};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Date {
    Epoch(u64),
    Local(String),
    Utc(String),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum MemoryStat {
    Bytes(u64),
    HumanReadable(String),
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub date: Date, //utc date in epoch seconds
    pub uptime: Option<u64>, //uptime in seconds
    pub boot_time: Option<Date>, //boot time in epoch
    pub mem: Option<MemoryInfo>,
    pub cpu_load: Option<CpuLoad>,
    pub load_avg: Option<CpuLoadAvg>,
    pub net_stats: Option<Vec<NetworkInfo>>,
    pub fs_stats: Option<Vec<FilesystemInfo>>,
    //socket stats
    //custom scripts
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total: MemoryStat,
    pub free: MemoryStat,
    pub percentage_used: f32,
}

#[derive(Serialize)]
pub struct CpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub interrupt: f32,
    pub idle: f32,
}

#[derive(Serialize)]
pub struct CpuLoadAvg {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub name: String, 
    pub stats: Option<NetworkStats>,
}

#[derive(Serialize)]
pub struct NetworkStats {
    pub rx_bytes: MemoryStat,
    pub tx_bytes: MemoryStat,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Serialize)]
pub struct FilesystemInfo {
    pub name: String,
    pub free: MemoryStat,
    pub avail: MemoryStat,
    pub total: MemoryStat,
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

pub struct StatsConfig {
    pub date_format: DateFormat,
    pub human_readable: bool,
}

pub struct StatsCollector<'a>  {
    pub cfg: &'a StatsConfig,
    pub sys: System,
}

impl<'a> StatsCollector<'a> {
    pub fn new(cfg: &StatsConfig) -> StatsCollector {
        StatsCollector {
            cfg: cfg,
            sys: System::new(),
        }
    }

    pub fn get_stats(&self) -> SystemInfo {
        SystemInfo {
            date: self.get_current_date(), //why is timestamp signed
            uptime: print_err(self.sys.uptime()).map(|d| d.as_secs()),
            boot_time: print_err(self.sys.boot_time()).map(|d| self.convert_to_date(d)),
            mem: self.get_mem_stats(),
            cpu_load: self.get_cpu_load(), 
            load_avg: self.get_cpu_load_avg(), 
            net_stats: self.get_network_stats(),
            fs_stats: self.get_fs_stats(),
        }
    }

    fn to_memorystat(&self, size: u64) -> MemoryStat {
        if self.cfg.human_readable {
            MemoryStat::HumanReadable((size/(1024*1024)).to_string() + "MiB") 
        } else {
            MemoryStat::Bytes(size)
        }
    }

    fn get_current_date(&self) -> Date {
        match self.cfg.date_format {
            DateFormat::Epoch => Date::Epoch(offset::Utc::now().timestamp() as u64),
            DateFormat::Local => Date::Local(offset::Local::now().to_string()),
            DateFormat::Utc => Date::Utc(offset::Utc::now().to_string()),
        }
    }

    fn convert_to_date(&self, date: DateTime<Utc>) -> Date {
        match self.cfg.date_format {
            DateFormat::Epoch => Date::Epoch(date.timestamp() as u64),
            DateFormat::Local => Date::Local(DateTime::<Local>::from(date).to_string()),
            DateFormat::Utc => Date::Utc(date.to_string()),
        }
    }

    fn get_mem_stats(&self) -> Option<MemoryInfo> {
        print_err(self.sys.memory()).map(|m| MemoryInfo {
            total: self.to_memorystat(m.total.as_u64()),
            free: self.to_memorystat(m.free.as_u64()),
            percentage_used: (((m.total.as_u64()-m.free.as_u64()) as f32)/(m.total.as_u64() as f32) * 100.0), 
            //todo: research if this is returns a valid, useful value
        })
    }

    fn get_cpu_load(&self) -> Option<CpuLoad> {
        let cpu_load = print_err(self.sys.cpu_load_aggregate()); //makes no sense right now, you need to wait like a second before you call done

        cpu_load.map(|c| c.done().unwrap()).map(|c| CpuLoad {
            user: c.user,
            nice: c.nice,
            system: c.system,
            interrupt: c.interrupt,
            idle: c.idle,
        })
    }

    fn get_cpu_load_avg(&self) -> Option<CpuLoadAvg> {
        print_err(self.sys.load_average()).map(|l| CpuLoadAvg {
            one: l.one,
            five: l.five,
            fifteen: l.fifteen,
        })
    }

    fn get_network_stats(&self) -> Option<Vec<NetworkInfo>> {
        print_err(self.sys.networks()).map(|networks| 
            networks.values().map(|network| 
                NetworkInfo {
                    name: network.name.clone(),
                    stats: print_err(self.sys.network_stats(&network.name)).map(|stats| NetworkStats {
                        rx_bytes: self.to_memorystat(stats.rx_bytes.as_u64()),
                        tx_bytes: self.to_memorystat(stats.tx_bytes.as_u64()),
                        rx_packets: stats.rx_packets,
                        tx_packets: stats.tx_packets,
                        rx_errors: stats.rx_errors,
                        tx_errors: stats.tx_errors,
                    })
                }
            ).collect::<Vec<NetworkInfo>>()
        )
    }

    fn get_fs_stats(&self) -> Option<Vec<FilesystemInfo>> {
        print_err(self.sys.mounts()).map(|mounts|
            mounts.iter().map(|mount| 
                FilesystemInfo {
                    name: mount.fs_mounted_from.clone(),
                    free: self.to_memorystat(mount.free.as_u64()),
                    avail: self.to_memorystat(mount.avail.as_u64()),
                    total: self.to_memorystat(mount.total.as_u64()),
                }
            ).collect::<Vec<FilesystemInfo>>()
        )
    }
}
