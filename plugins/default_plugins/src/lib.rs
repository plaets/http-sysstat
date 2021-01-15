pub mod plugin;

use http_sysstat_pluginlib::stats_collector::*;

//test
pub fn get_all() -> Vec<Box<dyn StatsCollector>> {
    plugin::get_all()
}
