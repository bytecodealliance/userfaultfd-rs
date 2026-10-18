use std::io;

use crate::IoctlFlags;
use nix::errno::Errno;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors for this crate.
///
/// Several of these errors contain an underlying `std::io::Error` value; see
/// [`userfaultfd(2)`](http://man7.org/linux/man-pages/man2/userfaultfd.2.html) and
/// [`ioctl_userfaultfd(2)`](http://man7.org/linux/man-pages/man2/ioctl_userfaultfd.2.html) for more
/// details on how to interpret these errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Copy ioctl failure with `errno` value.
    #[error("Copy failed")]
    CopyFailed(#[source] io::Error),

    /// Copy ioctl failure with copied length.
    #[error("Copy partially succeeded")]
    PartiallyCopied(usize),

    /// Zeropage ioctl failure with zeroed length.
    #[error("Zeropage partially succeeded")]
    PartiallyZeroed(usize),

    /// Failure to read a full `uffd_msg` struct from the underlying file descriptor.
    #[error("Incomplete uffd_msg; read only {read}/{expected} bytes")]
    IncompleteMsg { read: usize, expected: usize },

    /// Generic system error.
    #[error("System error")]
    SystemError(#[source] io::Error),

    /// End-of-file was read from the underlying file descriptor.
    #[error("EOF when reading file descriptor")]
    ReadEof,

    /// An unrecognized event code was found in a `uffd_msg` struct.
    #[error("Unrecognized event in uffd_msg: {0}")]
    UnrecognizedEvent(u8),

    /// Requested ioctls were not available when initializing the API.
    #[error("Requested ioctls unsupported; supported: {0:?}")]
    UnsupportedIoctls(IoctlFlags),

    /// Zeropage ioctl failure with `errno` value.
    #[error("Zeropage failed")]
    ZeropageFailed(#[source] io::Error),

    /// Could not open /dev/userfaultfd even though it exists
    #[error("Error accessing /dev/userfaultfd")]
    OpenDevUserfaultfd(#[source] io::Error),
}

impl Error {
    /// Converts an `Errno` returned by a system call into `Error::SystemError`.
    ///
    /// This is deliberately not a `From<Errno>` implementation: trait implementations are always
    /// public, and that would make `nix` part of this crate's public API.
    pub(crate) fn sys(errno: Errno) -> Self {
        Error::SystemError(io::Error::from(errno))
    }
}
