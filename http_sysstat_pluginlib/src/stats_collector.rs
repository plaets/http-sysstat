use serde::{Serialize, Deserialize};
use serde_json::{Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    Epoch,
    Local,
    Utc,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ConfigValue {
    None,
    Bool(bool),
    Number(u8),
    String(String),
    List(Vec<ConfigValue>),
    Map(HashMap<String, ConfigValue>),
}

pub struct StatsConfig {
    pub date_format: DateFormat,
    pub human_readable: bool,
    pub query_other: HashMap<String, String>, //other http query parameters
    pub plugin_config: Arc<HashMap<String, ConfigValue>>, //whole plugin_config section from the config file
}

pub trait StatsCollector: Send {
    //constructor
    fn new(/* program config */) -> Self where Self: Sized;
    //called during a request
    fn collect(&self, config: &StatsConfig) -> serde_json::Result<Value>;
    //should return name of the plugin
    fn name(&self) -> &'static str;
}
