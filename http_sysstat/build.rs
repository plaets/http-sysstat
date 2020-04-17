use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("{}", "cargo:rustc-link-search=".to_owned() + env::var_os("OUT_DIR").unwrap().to_str().unwrap() + "/../../../"); //this WILL stop working one day

    let main_dir_var = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let main_dir = main_dir_var.to_str().unwrap();

    let mut plugins_dir = fs::read_dir(Path::new(&main_dir).join("..").join("plugins")).unwrap();

    let mut s: String = String::from(plugins_dir.by_ref().fold(String::new(), |s, dir| s + &"extern crate " + dir.unwrap().file_name().to_str().unwrap() + ";\n"));

    plugins_dir = fs::read_dir(Path::new(&main_dir).join("..").join("plugins")).unwrap();
    s += &(String::from("fn get_all() -> Vec<Box<StatsCollector>> {\nlet mut plugins = vec!();\n") + plugins_dir.by_ref().fold(String::new(), |s, dir| s + "plugins.extend(" + dir.unwrap().file_name().to_str().unwrap() + "::get_all()); \n").as_str() + ";\nplugins\n}\n");
    
    let out_dir_var = env::var_os("OUT_DIR").unwrap();
    let out_dir = out_dir_var.to_str().unwrap();
    let out = Path::new(&out_dir).join("plugins.rs");
    fs::write(out.clone(), s).unwrap();

    //TODO: refactoring, use templates instead of building strings
    //TODO: docs
    //TODO: better api
}
