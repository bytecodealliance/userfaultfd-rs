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
    let mut bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // filter out stuff from <linux/types.h>
        .blocklist_item("__BITS_PER_LONG")
        .blocklist_item("__FD_SETSIZE")
        .blocklist_type("__[lb]e.*")
        .blocklist_type("__w?sum.*")
        .blocklist_type("__kernel_*")
        .parse_callbacks(Box::new(Callbacks {}));

    if let Ok(linux_headers) = std::env::var("LINUX_HEADERS") {
        let mut incl_dir = PathBuf::from(&linux_headers);
        incl_dir.push("include");
        assert!(
            incl_dir.exists(),
            "LINUX_HEADERS env variable contains an include/ directory"
        );
        incl_dir.push("uapi");
        assert!(
            incl_dir.exists(),
            "LINUX_HEADERS env variable contains an include/uapi/ directory"
        );
        bindings = bindings
            .clang_arg(format!("-isystem{}/include", linux_headers))
            .clang_arg(format!("-isystem{}/include/uapi", linux_headers));
    }

    let bindings = bindings.generate().expect("binding generation failed");

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
