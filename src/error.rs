use crate::IoctlFlags;
use failure::Fail;
use nix::errno::Errno;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors for this crate.
///
/// Several of these errors contain an underlying `Errno` value; see
/// [`userfaultfd(2)`](http://man7.org/linux/man-pages/man2/userfaultfd.2.html) and
/// [`ioctl_userfaultfd(2)`](http://man7.org/linux/man-pages/man2/ioctl_userfaultfd.2.html) for more
/// details on how to interpret these errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Copy ioctl failure with `errno` value.
    #[fail(display = "copy failed: {}", 0)]
    CopyFailed(Errno),

    /// Failure to read a full `uffd_msg` struct from the underlying file descriptor.
    #[fail(display = "incomplete uffd_msg; read only {}/{} bytes", read, expected)]
    IncompleteMsg { read: usize, expected: usize },

    /// Generic system error.
    #[fail(display = "system error: {}", 0)]
    SystemError(nix::Error),

    /// End-of-file was read from the underlying file descriptor.
    #[fail(display = "EOF when reading file descriptor")]
    ReadEof,

    /// An unrecognized event code was found in a `uffd_msg` struct.
    #[fail(display = "unrecognized event in uffd_msg: {}", 0)]
    UnrecognizedEvent(u8),

    /// An unrecognized ioctl bit was set in the result of API initialization or registration.
    #[fail(display = "unrecognized ioctl flags: {}", 0)]
    UnrecognizedIoctls(u64),

    /// Requested ioctls were not available when initializing the API.
    #[fail(display = "requested ioctls unsupported; supported: {:?}", 0)]
    UnsupportedIoctls(IoctlFlags),

    /// Zeropage ioctl failure with `errno` value.
    #[fail(display = "zeropage failed: {}", 0)]
    ZeropageFailed(Errno),
}

impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Error {
        Error::SystemError(e)
    }
}
