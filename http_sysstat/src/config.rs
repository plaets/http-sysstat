use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;

use http_sysstat_pluginlib::stats_collector::{ConfigValue};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub addr: String,
    pub plugin_config: Arc<HashMap<String, ConfigValue>>,
}

/* config:
 * cache
 * password protection
 * should warn if plugin specified in config does not exist
 */

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8080"),
            plugin_config: Arc::new(HashMap::new()),
        }
    }
}
