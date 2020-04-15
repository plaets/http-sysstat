use serde_json::{Value, json};
use chrono::{offset};

use http_sysstat_pluginlib::stats_collector::*;
use http_sysstat_pluginlib::utils::*;

pub struct TestPlugin;
impl StatsCollector for TestPlugin {
    fn new() -> Self { Self{} }

    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        serde_json::to_value(match cfg.date_format {
            DateFormat::Epoch => Date::Epoch(offset::Utc::now().timestamp() as u64),
            DateFormat::Local => Date::Local(offset::Local::now().to_string()),
            DateFormat::Utc => Date::Utc(offset::Utc::now().to_string()),
        })
    }

    fn name(&self) -> &'static str {
        "test"
    }
}
