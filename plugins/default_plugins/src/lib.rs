pub mod plugin;

use http_sysstat_pluginlib::stats_collector::*;

pub fn get_all() -> Vec<Box<dyn StatsCollector>> {
    plugin::get_all()
}
