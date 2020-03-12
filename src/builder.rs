use crate::error::{Error, Result};
use crate::raw;
use crate::{IoctlFlags, Uffd};
use bitflags::bitflags;
use nix::errno::Errno;

bitflags! {
    /// Used with `UffdBuilder` to determine which features are available in the current kernel.
    pub struct FeatureFlags: u64 {
        const PAGEFAULT_FLAG_WP = raw::UFFD_FEATURE_PAGEFAULT_FLAG_WP;
        const EVENT_FORK = raw::UFFD_FEATURE_EVENT_FORK;
        const EVENT_REMAP = raw::UFFD_FEATURE_EVENT_REMAP;
        const EVENT_REMOVE = raw::UFFD_FEATURE_EVENT_REMOVE;
        const MISSING_HUGETLBFS = raw::UFFD_FEATURE_MISSING_HUGETLBFS;
        const MISSING_SHMEM = raw::UFFD_FEATURE_MISSING_SHMEM;
        const EVENT_UNMAP = raw::UFFD_FEATURE_EVENT_UNMAP;
    }
}

/// A builder for initializing `Uffd` objects.
///
/// ```
/// use userfaultfd::UffdBuilder;
///
/// let uffd = UffdBuilder::new()
///     .close_on_exec(true)
///     .non_blocking(true)
///     .create();
/// assert!(uffd.is_ok());
/// ```
pub struct UffdBuilder {
    close_on_exec: bool,
    non_blocking: bool,
    req_features: FeatureFlags,
    req_ioctls: IoctlFlags,
}

impl UffdBuilder {
    /// Create a new builder with no required features or ioctls, and `close_on_exec` and
    /// `non_blocking` both set to `false`.
    pub fn new() -> UffdBuilder {
        UffdBuilder {
            close_on_exec: false,
            non_blocking: false,
            req_features: FeatureFlags::empty(),
            req_ioctls: IoctlFlags::empty(),
        }
    }

    /// Enable the close-on-exec flag for the new userfaultfd object (see the description of
    /// `O_CLOEXEC` in [`open(2)`](http://man7.org/linux/man-pages/man2/open.2.html)).
    pub fn close_on_exec(&mut self, close_on_exec: bool) -> &mut Self {
        self.close_on_exec = close_on_exec;
        self
    }

    /// Enable non-blocking operation for the userfaultfd object.
    ///
    /// If this is set to `false`, `Uffd::read_event()` will block until an event is available to
    /// read. Otherwise, it will immediately return `None` if no event is available.
    pub fn non_blocking(&mut self, non_blocking: bool) -> &mut Self {
        self.non_blocking = non_blocking;
        self
    }

    /// Add a requirement that a particular feature or set of features is available.
    ///
    /// If a required feature is unavailable, `UffdBuilder.create()` will return an error.
    pub fn require_features(&mut self, feature: FeatureFlags) -> &mut Self {
        self.req_features |= feature;
        self
    }

    /// Add a requirement that a particular ioctl or set of ioctls is available.
    ///
    /// If a required ioctl is unavailable, `UffdBuilder.create()` will return an error.
    pub fn require_ioctls(&mut self, ioctls: IoctlFlags) -> &mut Self {
        self.req_ioctls |= ioctls;
        self
    }

    /// Create a `Uffd` object with the current settings of this builder.
    pub fn create(&self) -> Result<Uffd> {
        // first do the syscall to get the file descriptor
        let mut flags = 0;
        if self.close_on_exec {
            flags |= libc::O_CLOEXEC;
        }
        if self.non_blocking {
            flags |= libc::O_NONBLOCK;
        }
        let fd = Errno::result(unsafe { raw::userfaultfd(flags) })?;

        // then do the UFFDIO_API ioctl to set up and ensure features and other ioctls are available
        let mut api = raw::uffdio_api {
            api: raw::UFFD_API,
            features: self.req_features.bits(),
            ioctls: 0,
        };
        unsafe {
            raw::api(fd, &mut api as *mut raw::uffdio_api)?;
        }
        let supported =
            IoctlFlags::from_bits(api.ioctls).ok_or(Error::UnrecognizedIoctls(api.ioctls))?;
        if !supported.contains(self.req_ioctls) {
            Err(Error::UnsupportedIoctls(supported))
        } else {
            Ok(Uffd { fd })
        }
    }
}
