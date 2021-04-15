use super::*;

// The following are preprocessor constants that bindgen can't figure out, so we enter them manually
// from <linux/userfaultfd.h>, and have tests to make sure they're accurate.

pub const UFFD_API: u64 = 0xAA;
pub const UFFD_API_FEATURES: u64 = UFFD_FEATURE_EVENT_FORK
    | UFFD_FEATURE_EVENT_REMAP
    | UFFD_FEATURE_EVENT_REMOVE
    | UFFD_FEATURE_EVENT_UNMAP
    | UFFD_FEATURE_MISSING_HUGETLBFS
    | UFFD_FEATURE_MISSING_SHMEM;

pub const UFFD_API_IOCTLS: u64 = 1 << _UFFDIO_REGISTER | 1 << _UFFDIO_UNREGISTER | 1 << _UFFDIO_API;
pub const UFFD_API_RANGE_IOCTLS: u64 =
    1 << _UFFDIO_WAKE | 1 << _UFFDIO_COPY | 1 << _UFFDIO_ZEROPAGE;

pub const UFFDIO_REGISTER_MODE_MISSING: u64 = 1 << 0;
pub const UFFDIO_REGISTER_MODE_WP: u64 = 1 << 1;

pub const UFFDIO_COPY_MODE_DONTWAKE: u64 = 1 << 0;

pub const UFFDIO_ZEROPAGE_MODE_DONTWAKE: u64 = 1 << 0;

#[cfg(test)]
mod const_tests {
    use super::*;

    extern "C" {
        static _const_UFFD_API: u64;
        static _const_UFFD_API_FEATURES: u64;
        static _const_UFFD_API_IOCTLS: u64;
        static _const_UFFD_API_RANGE_IOCTLS: u64;
        static _const_UFFDIO_REGISTER_MODE_MISSING: u64;
        static _const_UFFDIO_REGISTER_MODE_WP: u64;
        static _const_UFFDIO_COPY_MODE_DONTWAKE: u64;
        static _const_UFFDIO_ZEROPAGE_MODE_DONTWAKE: u64;
    }

    #[test]
    fn consts_correct() {
        unsafe {
            assert_eq!(UFFD_API, _const_UFFD_API, "UFFD_API");
            assert_eq!(
                UFFD_API_FEATURES, _const_UFFD_API_FEATURES,
                "UFFD_API_FEATURES"
            );
            assert_eq!(UFFD_API_IOCTLS, _const_UFFD_API_IOCTLS, "UFFD_API_IOCTLS");
            assert_eq!(
                UFFD_API_RANGE_IOCTLS, _const_UFFD_API_RANGE_IOCTLS,
                "UFFD_API_RANGE_IOCTLS"
            );
            assert_eq!(
                UFFDIO_REGISTER_MODE_MISSING, _const_UFFDIO_REGISTER_MODE_MISSING,
                "UFFDIO_REGISTER_MODE_MISSING"
            );
            assert_eq!(
                UFFDIO_REGISTER_MODE_WP, _const_UFFDIO_REGISTER_MODE_WP,
                "UFFDIO_REGISTER_MODE_WP"
            );
            assert_eq!(
                UFFDIO_COPY_MODE_DONTWAKE, _const_UFFDIO_COPY_MODE_DONTWAKE,
                "UFFDIO_COPY_MODE_DONTWAKE"
            );
            assert_eq!(
                UFFDIO_ZEROPAGE_MODE_DONTWAKE, _const_UFFDIO_ZEROPAGE_MODE_DONTWAKE,
                "UFFDIO_ZEROPAGE_MODE_DONTWAKE"
            );
        }
    }
}
