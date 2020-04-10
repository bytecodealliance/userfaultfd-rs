use bindgen;
use bindgen::callbacks::{IntKind, ParseCallbacks};
use cc;
use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings();

    cc::Build::new()
        .file("src/consts.c")
        .compile("userfaultfd_sys_consts");
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // filter out stuff from <linux/types.h>
        .blacklist_item("__BITS_PER_LONG")
        .blacklist_item("__FD_SETSIZE")
        .blacklist_type("__[lb]e.*")
        .blacklist_type("__w?sum.*")
        .blacklist_type("__kernel_.*")
        .parse_callbacks(Box::new(Callbacks {}))
        .generate()
        .expect("binding generation failed");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("binding file couldn't be written");
}

// all this stuff with callbacks is to give the integer constants the right types

#[derive(Debug)]
struct Callbacks {}

impl ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        for (prefix, kind) in [
            ("_UFFDIO_", IntKind::U64),
            ("UFFD_API", IntKind::U64),
            ("UFFDIO", IntKind::U8),
            ("UFFD_EVENT_", IntKind::U8),
            ("UFFD_PAGEFAULT_FLAG_", IntKind::U64),
            ("UFFD_FEATURE_", IntKind::U64),
        ]
        .iter()
        {
            if name.starts_with(prefix) {
                return Some(*kind);
            }
        }
        return None;
    }
}
