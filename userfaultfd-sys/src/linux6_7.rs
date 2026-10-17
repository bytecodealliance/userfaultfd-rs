use super::*;

pub use linux6_4::*;

pub const UFFD_API_FEATURES: u64 = linux6_4::UFFD_API_FEATURES | UFFD_FEATURE_WP_ASYNC;
