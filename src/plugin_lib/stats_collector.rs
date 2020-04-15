use serde::{Deserialize};
use serde_json::{Value};
use std::collections::HashMap;

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    Epoch,
    Local,
    Utc,
}

pub struct StatsConfig {
    pub date_format: DateFormat,
    pub human_readable: bool,
    pub query_other: HashMap<String, String>,
}

pub trait StatsCollector: Send {
    fn new(/* program config */) -> Self where Self: Sized;
    fn collect(&self, config: &StatsConfig) -> serde_json::Result<Value>;
    fn name(&self) -> &'static str;
}
