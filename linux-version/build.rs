use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .whitelist_var("LINUX_VERSION_CODE")
        .generate()
        .expect("binding generation failed");
    println!("rerun-if-changed=/usr/include/linux/version.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("binding file couldn't be written");
}
