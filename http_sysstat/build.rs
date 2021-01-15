use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

extern crate tera; 
use tera::{Tera, Context}; 

static PLUGINS_RS: &str = r#"
use http_sysstat_pluginlib::stats_collector::StatsCollector;

{% for crate in plugin_crates %}
extern crate {{ crate }};
{% endfor %}

fn get_all() -> Vec<Box<StatsCollector>> {
    let mut plugins = vec!();
    {% for crate in plugin_crates %}
    plugins.extend({{ crate }}::get_all());
    {% endfor %}
    plugins
}
"#;

fn build_plugins_rs(output_path: &Path, plugin_crates: &Vec<String>) {
    let mut tera = Tera::default();
    tera.add_raw_template("plugins.rs", PLUGINS_RS);

    let mut context = Context::new();
    context.insert("plugin_crates", &plugin_crates);

    fs::write(output_path, tera.render("plugins.rs", &context).unwrap());
}

fn print_rerun_paths(main_dir: &Path) {
    WalkDir::new(Path::new(&main_dir).join("../").join("plugins"))
        .into_iter()
        .filter_map(|e| e.ok()) 
        .for_each(|entry| println!("cargo:rerun-if-changed={}", entry.path().display()));
}

fn main() {
    println!("{}", "cargo:rustc-link-search=".to_owned() + env::var_os("OUT_DIR").unwrap().to_str().unwrap() + "/../../../"); //this WILL stop working one day

    let main_dir = &env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let main_dir_path = Path::new(main_dir);

    //make sure that the bulid script is rerun if any file in the repo is changed
    //not sure if that's necessary
    print_rerun_paths(&main_dir_path);

    //get all crates in "./plugins"
    //WARNING: ASSUMES THAT "./plugins" IS IN THE PARENT DIRECTORY OF THE `http_sysstat` CRATE
    let plugin_crates = fs::read_dir(main_dir_path.join("..").join("plugins"))
                .unwrap().map(|dir| String::from(dir.unwrap().file_name().to_str().unwrap())).collect::<Vec<String>>();

    //get the output directory
    let out_dir_var = env::var_os("OUT_DIR").unwrap();
    let out = Path::new(out_dir_var.to_str().unwrap()).join("plugins.rs");

    //generate plugins.rs
    build_plugins_rs(&out, &plugin_crates);

    //TODO: error handling/messages
}
