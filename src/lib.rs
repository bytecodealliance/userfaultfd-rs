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
pub use crate::event::{Event, FaultKind, ReadWrite};

use bitflags::bitflags;
use libc::{self, c_void};
use nix::errno::Errno;
use nix::unistd::read;
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

/// Represents an opaque buffer where userfaultfd events are stored.
///
/// This is used in conjunction with [`Uffd::read_events`].
pub struct EventBuffer(Vec<raw::uffd_msg>);

impl EventBuffer {
    /// Creates a new buffer for `size` number of events.
    ///
    /// [`Uffd::read_events`] will read up to this many events at a time.
    pub fn new(size: usize) -> Self {
        Self(vec![unsafe { mem::zeroed() }; size])
    }
}

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

bitflags! {
    /// The registration mode used when registering an address range with `Uffd`.
    pub struct RegisterMode: u64 {
        /// Registers the range for missing page faults.
        const MISSING = raw::UFFDIO_REGISTER_MODE_MISSING;
        /// Registers the range for write faults.
        #[cfg(feature = "linux5_7")]
        const WRITE_PROTECT = raw::UFFDIO_REGISTER_MODE_WP;
    }
}

impl Uffd {
    /// Register a memory address range with the userfaultfd object, and returns the `IoctlFlags`
    /// that are available for the selected range.
    ///
    /// This method only registers the given range for missing page faults.
    pub fn register(&self, start: *mut c_void, len: usize) -> Result<IoctlFlags> {
        self.register_with_mode(start, len, RegisterMode::MISSING)
    }

    /// Register a memory address range with the userfaultfd object for the given mode and
    /// returns the `IoctlFlags` that are available for the selected range.
    pub fn register_with_mode(
        &self,
        start: *mut c_void,
        len: usize,
        mode: RegisterMode,
    ) -> Result<IoctlFlags> {
        let mut register = raw::uffdio_register {
            range: raw::uffdio_range {
                start: start as u64,
                len: len as u64,
            },
            mode: mode.bits(),
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

    /// Atomically copy a continuous memory chunk into the userfaultfd-registered range, and return
    /// the number of bytes that were successfully copied.
    ///
    /// If `wake` is `true`, wake up the thread waiting for page fault resolution on the memory
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

        let _ = raw::copy(self.as_raw_fd(), &mut copy as *mut raw::uffdio_copy)
            .map_err(Error::CopyFailed)?;
        if copy.copy < 0 {
            // shouldn't ever get here, as errno should be caught above
            Err(Error::CopyFailed(Errno::from_i32(-copy.copy as i32)))
        } else {
            Ok(copy.copy as usize)
        }
    }

    /// Zero out a memory address range registered with userfaultfd, and return the number of bytes
    /// that were successfully zeroed.
    ///
    /// If `wake` is `true`, wake up the thread waiting for page fault resolution on the memory
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

        let _ = raw::zeropage(self.as_raw_fd(), &mut zeropage as &mut raw::uffdio_zeropage)
            .map_err(Error::ZeropageFailed)?;
        if zeropage.zeropage < 0 {
            // shouldn't ever get here, as errno should be caught above
            Err(Error::ZeropageFailed(Errno::from_i32(
                -zeropage.zeropage as i32,
            )))
        } else {
            Ok(zeropage.zeropage as usize)
        }
    }

    /// Wake up the thread waiting for page fault resolution on the specified memory address range.
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

    /// Makes a range write-protected.
    #[cfg(feature = "linux5_7")]
    pub fn write_protect(&self, start: *mut c_void, len: usize) -> Result<()> {
        let mut ioctl = raw::uffdio_writeprotect {
            range: raw::uffdio_range {
                start: start as u64,
                len: len as u64,
            },
            mode: raw::UFFDIO_WRITEPROTECT_MODE_WP,
        };

        unsafe {
            raw::write_protect(
                self.as_raw_fd(),
                &mut ioctl as *mut raw::uffdio_writeprotect,
            )?;
        }

        Ok(())
    }

    /// Removes the write-protection for a range.
    ///
    /// If `wake` is `true`, wake up the thread waiting for page fault resolution on the memory
    /// address range.
    #[cfg(feature = "linux5_7")]
    pub fn remove_write_protection(
        &self,
        start: *mut c_void,
        len: usize,
        wake: bool,
    ) -> Result<()> {
        let mut ioctl = raw::uffdio_writeprotect {
            range: raw::uffdio_range {
                start: start as u64,
                len: len as u64,
            },
            mode: if wake {
                0
            } else {
                raw::UFFDIO_WRITEPROTECT_MODE_DONTWAKE
            },
        };

        unsafe {
            raw::write_protect(
                self.as_raw_fd(),
                &mut ioctl as *mut raw::uffdio_writeprotect,
            )?;
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use userfaultfd::{Uffd, Result};
    /// fn read_event(uffd: &Uffd) -> Result<()> {
    ///     // Read a single event
    ///     match uffd.read_event()? {
    ///         Some(e) => {
    ///             // Do something with the event
    ///         },
    ///         None => {
    ///             // This was a non-blocking read and the descriptor was not ready for read
    ///         },
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn read_event(&self) -> Result<Option<Event>> {
        let mut buf = [unsafe { std::mem::zeroed() }; 1];
        let mut iter = self.read(&mut buf)?;
        let event = iter.next().transpose()?;
        assert!(iter.next().is_none());
        Ok(event)
    }

    /// Read multiple events from the userfaultfd object using the given event buffer.
    ///
    /// If the `Uffd` object was created with `non_blocking` set to `false`, this will block until
    /// an event is successfully read or an error is returned.
    ///
    /// If `non_blocking` was `true`, this will immediately return an empty iterator if the file
    /// descriptor is not ready for reading.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use userfaultfd::{Uffd, EventBuffer};
    /// fn read_events(uffd: &Uffd) -> userfaultfd::Result<()> {
    ///     // Read up to 100 events at a time
    ///     let mut buf = EventBuffer::new(100);
    ///     for event in uffd.read_events(&mut buf)? {
    ///         let event = event?;
    ///         // Do something with the event...
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn read_events<'a>(
        &self,
        buf: &'a mut EventBuffer,
    ) -> Result<impl Iterator<Item = Result<Event>> + 'a> {
        self.read(&mut buf.0)
    }

    fn read<'a>(
        &self,
        msgs: &'a mut [raw::uffd_msg],
    ) -> Result<impl Iterator<Item = Result<Event>> + 'a> {
        const MSG_SIZE: usize = std::mem::size_of::<raw::uffd_msg>();

        let buf = unsafe {
            std::slice::from_raw_parts_mut(msgs.as_mut_ptr() as _, msgs.len() * MSG_SIZE)
        };

        let count = match read(self.as_raw_fd(), buf) {
            Err(e) if e == Errno::EAGAIN => 0,
            Err(e) => return Err(Error::SystemError(e)),
            Ok(0) => return Err(Error::ReadEof),
            Ok(bytes_read) => {
                let remainder = bytes_read % MSG_SIZE;
                if remainder != 0 {
                    return Err(Error::IncompleteMsg {
                        read: remainder,
                        expected: MSG_SIZE,
                    });
                }

                bytes_read / MSG_SIZE
            }
        };

        Ok(msgs.iter().take(count).map(|msg| Event::from_uffd_msg(msg)))
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
        #[cfg(feature = "linux5_7")]
        const WRITE_PROTECT = 1 << raw::_UFFDIO_WRITEPROTECT;
        const API = 1 << raw::_UFFDIO_API;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ptr;
    use std::thread;

    #[test]
    fn test_read_event() -> Result<()> {
        const PAGE_SIZE: usize = 4096;

        unsafe {
            let uffd = UffdBuilder::new().close_on_exec(true).create()?;

            let mapping = libc::mmap(
                ptr::null_mut(),
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );

            assert!(!mapping.is_null());

            uffd.register(mapping, PAGE_SIZE)?;

            let ptr = mapping as usize;
            let thread = thread::spawn(move || {
                let ptr = ptr as *mut u8;
                *ptr = 1;
            });

            match uffd.read_event()? {
                Some(Event::Pagefault {
                    rw: ReadWrite::Write,
                    addr,
                    ..
                }) => {
                    assert_eq!(addr, mapping);
                    uffd.zeropage(addr, PAGE_SIZE, true)?;
                }
                _ => panic!("unexpected event"),
            }

            thread.join().expect("failed to join thread");

            uffd.unregister(mapping, PAGE_SIZE)?;

            assert_eq!(libc::munmap(mapping, PAGE_SIZE), 0);
        }

        Ok(())
    }

    #[test]
    fn test_nonblocking_read_event() -> Result<()> {
        const PAGE_SIZE: usize = 4096;

        unsafe {
            let uffd = UffdBuilder::new()
                .close_on_exec(true)
                .non_blocking(true)
                .create()?;

            let mapping = libc::mmap(
                ptr::null_mut(),
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );

            assert!(!mapping.is_null());

            uffd.register(mapping, PAGE_SIZE)?;

            assert!(uffd.read_event()?.is_none());

            let ptr = mapping as usize;
            let thread = thread::spawn(move || {
                let ptr = ptr as *mut u8;
                *ptr = 1;
            });

            loop {
                match uffd.read_event()? {
                    Some(Event::Pagefault {
                        rw: ReadWrite::Write,
                        addr,
                        ..
                    }) => {
                        assert_eq!(addr, mapping);
                        uffd.zeropage(addr, PAGE_SIZE, true)?;
                        break;
                    }
                    Some(_) => panic!("unexpected event"),
                    None => thread::sleep(std::time::Duration::from_millis(50)),
                }
            }

            thread.join().expect("failed to join thread");

            uffd.unregister(mapping, PAGE_SIZE)?;

            assert_eq!(libc::munmap(mapping, PAGE_SIZE), 0);
        }

        Ok(())
    }

    #[test]
    fn test_read_events() -> Result<()> {
        unsafe {
            const MAX_THREADS: usize = 5;
            const PAGE_SIZE: usize = 4096;
            const MEM_SIZE: usize = PAGE_SIZE * MAX_THREADS;

            let uffd = UffdBuilder::new().close_on_exec(true).create()?;

            let mapping = libc::mmap(
                ptr::null_mut(),
                MEM_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );

            assert!(!mapping.is_null());

            uffd.register(mapping, MEM_SIZE)?;

            // As accessing the memory will suspend each thread with a page fault event,
            // there is no way to signal that the operations the test thread is waiting on to
            // complete have been performed.
            //
            // Therefore, this is inherently racy. The best we can do is simply sleep-wait for
            // all threads to have signaled that the operation is *about to be performed*.
            let mut seen = [false; MAX_THREADS];
            let mut threads = Vec::new();
            for i in 0..MAX_THREADS {
                let seen = &mut seen[i] as *mut _ as usize;
                let ptr = (mapping as *mut u8).add(PAGE_SIZE * i) as usize;
                threads.push(thread::spawn(move || {
                    let seen = seen as *mut bool;
                    let ptr = ptr as *mut u8;
                    *seen = true;
                    *ptr = 1;
                }));
            }

            loop {
                // Sleep even if all threads have "signaled", just in case any
                // thread is preempted prior to faulting the memory access.
                // Still, there's no guarantee that the call to `read_events` below will
                // read all the events at once, but this should be "good enough".
                let done = seen.iter().all(|b| *b);
                thread::sleep(std::time::Duration::from_millis(50));
                if done {
                    break;
                }
            }

            // Read all the events at once
            let mut buf = EventBuffer::new(MAX_THREADS);
            let mut iter = uffd.read_events(&mut buf)?;

            let mut seen = [false; MAX_THREADS];
            for _ in 0..MAX_THREADS {
                match iter
                    .next()
                    .transpose()?
                    .expect("failed to read all events; potential race condition was hit")
                {
                    Event::Pagefault {
                        rw: ReadWrite::Write,
                        addr,
                        ..
                    } => {
                        let index = (addr as usize - mapping as usize) / PAGE_SIZE;
                        assert_eq!(seen[index], false);
                        seen[index] = true;
                        uffd.zeropage(addr, PAGE_SIZE, true)?;
                    }
                    _ => panic!("unexpected event"),
                }
            }

            assert!(seen.iter().all(|b| *b));

            for thread in threads {
                thread.join().expect("failed to join thread");
            }

            uffd.unregister(mapping, MEM_SIZE)?;

            assert_eq!(libc::munmap(mapping, MEM_SIZE), 0);
        }

        Ok(())
    }

    #[cfg(feature = "linux5_7")]
    #[test]
    fn test_write_protect() -> Result<()> {
        const PAGE_SIZE: usize = 4096;

        unsafe {
            let uffd = UffdBuilder::new()
                .require_features(FeatureFlags::PAGEFAULT_FLAG_WP)
                .close_on_exec(true)
                .create()?;

            let mapping = libc::mmap(
                ptr::null_mut(),
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );

            assert!(!mapping.is_null());

            // This test uses both missing and write-protect modes for a reason.
            // The `uffdio_writeprotect` ioctl can only be used on a range *after*
            // the missing fault is handled, it seems. This means we either need to
            // read/write the page *before* we protect it or handle the missing
            // page fault by changing the protection level *after* we zero the page.
            assert!(uffd
                .register_with_mode(
                    mapping,
                    PAGE_SIZE,
                    RegisterMode::MISSING | RegisterMode::WRITE_PROTECT
                )?
                .contains(IoctlFlags::WRITE_PROTECT));

            let ptr = mapping as usize;
            let thread = thread::spawn(move || {
                let ptr = ptr as *mut u8;
                *ptr = 1;
                *ptr = 2;
            });

            loop {
                match uffd.read_event()? {
                    Some(Event::Pagefault {
                        kind,
                        rw: ReadWrite::Write,
                        addr,
                        ..
                    }) => match kind {
                        FaultKind::WriteProtected => {
                            assert_eq!(addr, mapping);
                            assert_eq!(*(addr as *const u8), 0);
                            // Remove the protection and wake the page
                            uffd.remove_write_protection(mapping, PAGE_SIZE, true)?;
                            break;
                        }
                        FaultKind::Missing => {
                            assert_eq!(addr, mapping);
                            uffd.zeropage(mapping, PAGE_SIZE, false)?;

                            // Technically, we already know it was a write that triggered
                            // the missing page fault, so there's little point in immediately
                            // write-protecting the page to cause another fault; in the real
                            // world, a missing fault with `rw` being `ReadWrite::Write` would
                            // be enough to mark the page as "dirty". For this test, however,
                            // we do it this way to ensure a write-protected fault is read.
                            assert_eq!(*(addr as *const u8), 0);
                            uffd.write_protect(mapping, PAGE_SIZE)?;
                            uffd.wake(mapping, PAGE_SIZE)?;
                        }
                    },
                    _ => panic!("unexpected event"),
                }
            }

            thread.join().expect("failed to join thread");

            assert_eq!(*(mapping as *const u8), 2);

            uffd.unregister(mapping, PAGE_SIZE)?;

            assert_eq!(libc::munmap(mapping, PAGE_SIZE), 0);
        }

        Ok(())
    }
}
