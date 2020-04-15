use serde::{Serialize};
use chrono::{Utc, Local, DateTime};
use std::io::{Result};

use crate::plugin_lib::stats_collector::{DateFormat, StatsConfig};

pub fn print_err<T>(r: Result<T>) -> Option<T> { //TODO: should take config and print to logs instead (rename to log_err for ex.)
    match r {
        Ok(val) => Some(val),
        Err(err) => {
            println!("error: {:?}", err);
            None
        }
    }
}

pub fn to_memorysize(cfg: &StatsConfig, size: u64) -> MemorySize {
    //TODO: handle more units than mib
    if cfg.human_readable {
        MemorySize::HumanReadable((size/(1024*1024)).to_string() + "MiB") 
    } else {
        MemorySize::Bytes(size)
    }
}

pub fn convert_to_date(cfg: &StatsConfig, date: DateTime<Utc>) -> Date { //maybe should only take datetype, not the whole config
   match cfg.date_format {
       DateFormat::Epoch => Date::Epoch(date.timestamp() as u64),
       DateFormat::Local => Date::Local(DateTime::<Local>::from(date).to_string()),
       DateFormat::Utc => Date::Utc(date.to_string()),
   }
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
