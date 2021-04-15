//! System bindings to `userfaultfd`.
//!
//! The minimum supported Linux kernel version is 4.11, but additional features from 4.14+ are
//! available by using the `linux4_14` Cargo feature.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "linux5_7")] {
        mod linux5_7;
        pub use crate::linux5_7::*;
    }
    else if #[cfg(feature = "linux4_14")] {
        mod linux4_14;
        pub use crate::linux4_14::*;
    } else {
        mod linux4_11;
        pub use crate::linux4_11::*;
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
