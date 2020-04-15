use std::env;

fn main() {
    println!("{}", "cargo:rustc-link-search=".to_owned() + env::var_os("OUT_DIR").unwrap().to_str().unwrap() + "/../../../"); //this WILL stop working one day
}
