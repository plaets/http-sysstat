use std::sync::{Arc, Mutex};
use confy;

mod server;
use server::start_server;

mod config;
use config::Config;

include!(concat!(env!("OUT_DIR"), "/plugins.rs"));

fn main() -> Result<(), confy::ConfyError> {
    let plugins = Arc::new(Mutex::new(get_all()));
    let config: Config = confy::load_path("./http_sysstat.cfg")?;
    start_server(config, plugins);
    Ok(())
}
