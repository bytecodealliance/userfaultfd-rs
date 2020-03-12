#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(linux4_14)] {
        mod linux4_14;
        pub use crate::linux4_14::*;
    } else if #[cfg(linux4_11)] {
        mod linux4_11;
        pub use crate::linux4_11::*;
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
