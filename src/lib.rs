//! A Linux mechanism for handling page faults in user space.
//!
//! The main way to interact with this library is to create a `Uffd` object with a `UffdBuilder`,
//! then use the methods of `Uffd` from a worker thread.
//!
//! See [`userfaultfd(2)`](http://man7.org/linux/man-pages/man2/userfaultfd.2.html) and
//! [`ioctl_userfaultfd(2)`](http://man7.org/linux/man-pages/man2/ioctl_userfaultfd.2.html) for more
//! details.

mod builder;
mod error;
mod event;
mod raw;

pub use crate::builder::{FeatureFlags, UffdBuilder};
pub use crate::error::{Error, Result};
pub use crate::event::Event;

use bitflags::bitflags;
use libc::{self, c_void};
use nix::errno::Errno;
use nix::unistd::read;
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

/// The userfaultfd object.
///
/// The userspace representation of the object is a file descriptor, so this type implements
/// `AsRawFd`, `FromRawFd`, and `IntoRawFd`. These methods should be used with caution, but can be
/// essential for using functions like `poll` on a worker thread.
#[derive(Debug)]
pub struct Uffd {
    fd: RawFd,
}

impl Drop for Uffd {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl AsRawFd for Uffd {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for Uffd {
    fn into_raw_fd(self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for Uffd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Uffd { fd }
    }
}

impl Uffd {
    /// Register a memory address range with the userfaultfd object, and returns the `IoctlFlags`
    /// that are available for the selected range.
    ///
    /// While the underlying `ioctl` call accepts mode flags, only one mode
    /// (`UFFDIO_REGISTER_MODE_MISSING`) is currently supported.
    pub fn register(&self, start: *mut c_void, len: usize) -> Result<IoctlFlags> {
        let mut register = raw::uffdio_register {
            range: raw::uffdio_range {
                start: start as u64,
                len: len as u64,
            },
            // this is the only mode currently supported
            mode: raw::UFFDIO_REGISTER_MODE_MISSING,
            ioctls: 0,
        };
        unsafe {
            raw::register(self.as_raw_fd(), &mut register as *mut raw::uffdio_register)?;
        }
        IoctlFlags::from_bits(register.ioctls).ok_or(Error::UnrecognizedIoctls(register.ioctls))
    }

    /// Unregister a memory address range from the userfaultfd object.
    pub fn unregister(&self, start: *mut c_void, len: usize) -> Result<()> {
        let mut range = raw::uffdio_range {
            start: start as u64,
            len: len as u64,
        };
        unsafe {
            raw::unregister(self.as_raw_fd(), &mut range as *mut raw::uffdio_range)?;
        }
        Ok(())
    }

    /// Atomically copy a continuous memory chunk into the userfaultfd-registed range, and return
    /// the number of bytes that were successfully copied.
    ///
    /// If `wake` is `true`, wake up the thread waiting for pagefault resolution on the memory
    /// range.
    pub unsafe fn copy(
        &self,
        src: *const c_void,
        dst: *mut c_void,
        len: usize,
        wake: bool,
    ) -> Result<usize> {
        let mut copy = raw::uffdio_copy {
            src: src as u64,
            dst: dst as u64,
            len: len as u64,
            mode: if wake {
                0
            } else {
                raw::UFFDIO_COPY_MODE_DONTWAKE
            },
            copy: 0,
        };

        let res = raw::copy(self.as_raw_fd(), &mut copy as *mut raw::uffdio_copy);
        if let Err(e) = res {
            if let Some(errno) = e.as_errno() {
                Err(Error::CopyFailed(errno))
            } else {
                Err(e.into())
            }
        } else {
            if copy.copy < 0 {
                // shouldn't ever get here, as errno should be caught above
                Err(Error::CopyFailed(Errno::from_i32(-copy.copy as i32)))
            } else {
                Ok(copy.copy as usize)
            }
        }
    }

    /// Zero out a memory address range registered with userfaultfd, and return the number of bytes
    /// that were successfully zeroed.
    ///
    /// If `wake` is `true`, wake up the thread waiting for pagefault resolution on the memory
    /// address range.
    pub unsafe fn zeropage(&self, start: *mut c_void, len: usize, wake: bool) -> Result<usize> {
        let mut zeropage = raw::uffdio_zeropage {
            range: raw::uffdio_range {
                start: start as u64,
                len: len as u64,
            },
            mode: if wake {
                0
            } else {
                raw::UFFDIO_ZEROPAGE_MODE_DONTWAKE
            },
            zeropage: 0,
        };

        let res = raw::zeropage(self.as_raw_fd(), &mut zeropage as &mut raw::uffdio_zeropage);
        if let Err(e) = res {
            if let Some(errno) = e.as_errno() {
                Err(Error::ZeropageFailed(errno))
            } else {
                Err(e.into())
            }
        } else {
            if zeropage.zeropage < 0 {
                // shouldn't ever get here, as errno should be caught above
                Err(Error::ZeropageFailed(Errno::from_i32(
                    -zeropage.zeropage as i32,
                )))
            } else {
                Ok(zeropage.zeropage as usize)
            }
        }
    }

    /// Wake up the thread waiting for pagefault resolution on the specified memory address range.
    pub fn wake(&self, start: *mut c_void, len: usize) -> Result<()> {
        let mut range = raw::uffdio_range {
            start: start as u64,
            len: len as u64,
        };
        unsafe {
            raw::wake(self.as_raw_fd(), &mut range as *mut raw::uffdio_range)?;
        }
        Ok(())
    }

    /// Read an `Event` from the userfaultfd object.
    ///
    /// If the `Uffd` object was created with `non_blocking` set to `false`, this will block until
    /// an event is successfully read (returning `Some(event)`, or an error is returned.
    ///
    /// If `non_blocking` was `true`, this will immediately return `None` if no event is ready to
    /// read.
    ///
    /// Note that while this method doesn't require a mutable reference to the `Uffd` object, it
    /// does consume bytes (thread-safely) from the underlying file descriptor.
    pub fn read_event(&self) -> Result<Option<Event>> {
        const MSG_SIZE: usize = mem::size_of::<raw::uffd_msg>();
        let mut buf = [0x00u8; MSG_SIZE];

        // read one uffd_msg at a time; maybe implement an iterator for handling many at once?
        let res = read(self.as_raw_fd(), &mut buf);

        match res {
            Err(e) if e.as_errno() == Some(Errno::EAGAIN) => return Ok(None),
            Err(e) => Err(e)?,
            Ok(nread) => {
                if nread == libc::EOF as usize {
                    return Err(Error::ReadEof);
                }
                if nread != MSG_SIZE {
                    return Err(Error::IncompleteMsg {
                        read: nread,
                        expected: MSG_SIZE,
                    });
                }
                let msg = unsafe { *(buf.as_mut_ptr() as *mut raw::uffd_msg) };
                let event = Event::from_uffd_msg(&msg)?;
                return Ok(Some(event));
            }
        }
    }
}

bitflags! {
    /// Used with `UffdBuilder` and `Uffd::register()` to determine which operations are available.
    pub struct IoctlFlags: u64 {
        const REGISTER = 1 << raw::_UFFDIO_REGISTER;
        const UNREGISTER = 1 << raw::_UFFDIO_UNREGISTER;
        const WAKE = 1 << raw::_UFFDIO_WAKE;
        const COPY = 1 << raw::_UFFDIO_COPY;
        const ZEROPAGE = 1 << raw::_UFFDIO_ZEROPAGE;
        const API = 1 << raw::_UFFDIO_API;
    }
}
