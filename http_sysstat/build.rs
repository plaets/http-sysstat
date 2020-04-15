fn main() {
    println!("cargo:rustc-link-lib=test-plugin");
    println!("cargo:rustc-link-search=crate=./plugins/test-plugin");
}
