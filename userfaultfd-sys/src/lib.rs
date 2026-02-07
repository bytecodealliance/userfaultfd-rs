//! System bindings to `userfaultfd`.
//!
//! The minimum supported Linux kernel version is 4.11, but additional features from 4.14+ are
//! available by using the `linux4_14` Cargo feature.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cfg_if::cfg_if;

mod linux4_11;
#[cfg(feature = "linux4_14")]
mod linux4_14;
#[cfg(feature = "linux5_7")]
mod linux5_7;

#[cfg(feature = "linux5_13")]
mod linux5_13;

#[cfg(feature = "linux6_4")]
mod linux6_4;

#[cfg(feature = "linux6_7")]
mod linux6_7;

cfg_if! {
    if #[cfg(feature = "linux6_7")] {
        pub use crate::linux6_7::*;
    } else if #[cfg(feature = "linux6_4")] {
        pub use crate::linux6_4::*;
    } else if #[cfg(feature = "linux5_13")] {
        pub use crate::linux5_13::*;
    } else if #[cfg(feature = "linux5_7")] {
        pub use crate::linux5_7::*;
    } else if #[cfg(feature = "linux4_14")] {
        pub use crate::linux4_14::*;
    } else {
        pub use crate::linux4_11::*;
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod const_tests {
    use super::*;

    extern "C" {
        static _const_UFFD_API_FEATURES: u64;
        static _const_UFFD_API_RANGE_IOCTLS: u64;
        static _const_UFFD_API_RANGE_IOCTLS_BASIC: u64;
    }

    #[test]
    fn consts_correct() {
        unsafe {
            assert_eq!(
                UFFD_API_FEATURES & _const_UFFD_API_FEATURES,
                UFFD_API_FEATURES,
                "UFFD_API_FEATURES"
            );
            assert_eq!(
                UFFD_API_RANGE_IOCTLS & _const_UFFD_API_RANGE_IOCTLS,
                UFFD_API_RANGE_IOCTLS,
                "UFFD_API_RANGE_IOCTLS"
            );
            assert_eq!(
                UFFD_API_RANGE_IOCTLS_BASIC & _const_UFFD_API_RANGE_IOCTLS_BASIC,
                UFFD_API_RANGE_IOCTLS_BASIC,
                "UFFD_API_RANGE_IOCTLS_BASIC"
            );
        }
    }
}
