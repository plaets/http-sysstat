use http_sysstat_pluginlib::stats_collector::*;
use http_sysstat_pluginlib::serde_json::{Value, Result};
use http_sysstat_pluginlib::serde_json;

pub struct ExampleCollector;
impl StatsCollector for ExampleCollector {
    fn new() -> Self { Self{} }

    //called during a http request, should return a serde_json value
    fn collect(&self, cfg: &StatsConfig) -> serde_json::Result<Value> {
        serde_json::to_value("test")
    }

    //name of the plugin
    fn name(&self) -> &'static str {
        "works"
    }
}

pub fn get_all() -> Vec<Box<dyn StatsCollector>> {
    vec!(
        Box::new(ExampleCollector::new())
    )
}
