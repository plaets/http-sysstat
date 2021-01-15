# http_sysstat

delivers system stats over http   
uses a (sorta cursed) compile time "plugin system"  

## crates

* http_sysstat - main crate, http server, configuration management
* http_sysstat_pluginlib - all plugins are supposed to import from this crate and implement the `StatsCollector` trait, includes some utilities
* plugins/default_plugins - set of default plugins that provide information about:
    + uptime
    + cpu load
    + ram usage
    + sockets
    + filesystems 
    + current time
    + network interfaces

## config

stored in `http_sysstat.cfg`   
example   
```toml
# ip address and port to listen on
addr = '127.0.0.1:8080'

# plugin specific configuration stored as a key-value map, passed to all plugins during a http request
[plugin_config]
```

## how to use it

* `cargo run` (run again in case of errors)
* `curl localhost:8080` (assuming that the example configuration is being used)

## adding a plugin

* create a library crate in `plugins`
* add the crate to the main `Cargo.toml` as a workspace
* add `http_sysstat_pluginlib = { path = "../../http_sysstat_pluginlib/" }` to dependencies of the new crate
* implement `get_plugins` in `lib.rs` of the new create (check `plugins/default_plugins` or `plugins/example` for examples)
* run cargo build twice - the first run might result in an error even if your code is valid

## plugin system

the most important part of the plugin system is implemented in `http_sysstat/build.rs`.   
first, `build.rs` finds all directories in `plugins`.    
it is assumed that every directory there is a valid crate that implements a `pub fn get_all() -> Vec<Box<dyn StatsCollector>>` function that is supposed to return all plugins from the crate.   
each plugin has to implement `http_sysstat_pluginlib::stats_collector::StatsCollector`.  
`build.rs` generates `plugins.rs` (to [`OUT_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)) which consists of a vector with structures implementing the plugins. this vector is used by `http_sysstat` as a plugin repository   
`plugins.rs` is then pasted into [`http_sysstat/src/main.rs`](http_sysstat/src/main.rs#L10)  

main issues with this system (that i remember):
* crates with plugins have to be added to the main `Cargo.toml` as a workspace member, otherwise you will get `error[E0463]: can't find crate for (...)`
* the build script assumes that `plugins` directory is in the parent directory of the `http_sysstat` crate. i'm not sure if this is guaranteed to be true, 
* `http_sysstat_pluginlib` has to be included in `Cargo.toml` of the plugins like this `http_sysstat_pluginlib = { path = "../../http_sysstat_pluginlib/" }` 
* sometimes you have to run `cargo build` twice for it to work, not sure why
* plugin names can clash resulting in one plugin replacing output of the other (this perhaps could be checked during runtime)
* dependency versions have to synchronized (and repeated) between `http_sysstat` and `http_sysstat_pluginlib`
* and probably much much more...

overall, this solution is shaky at best. it was a fun exercise in abusing build scripts, but i would not recommend to use it anywhere ever. 

## credits

this whole repo is inspired by `cargo-plugin`
[cargo-plugin](https://github.com/Geal/cargo-plugin)   


i have also discovered this crate and it looks pretty useful
[inventory](https://github.com/dtolnay/inventory)  

## examples of projects using static plugins/modules (for future reference)

* [telegraf](https://github.com/influxdata/telegraf/) - requires the user add paths of plugins to `all.go`
* [inspircd](https://github.com/inspircd/inspircd) - has a module manager
* [linux](https://github.com/torvalds/linux) 

