use crate::error::{Error, Result};
use crate::raw;
use crate::Uffd;
use libc::c_void;
#[cfg(linux4_14)]
use nix::unistd::Pid;
use std::os::unix::io::{FromRawFd, RawFd};

/// Whether a page fault event was for a read or write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadWrite {
    Read,
    Write,
}

/// The kind of fault for a page fault event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FaultKind {
    /// The fault was a read or write on a missing page.
    Missing,
    /// The fault was a write on a write-protected page.
    #[cfg(feature = "linux5_7")]
    WriteProtected,
}

/// Events from the userfaultfd object that are read by `Uffd::read_event()`.
#[derive(Debug)]
pub enum Event {
    /// A pagefault event.
    Pagefault {
        /// The kind of fault.
        kind: FaultKind,
        /// Whether the fault is on a read or a write.
        rw: ReadWrite,
        /// The address that triggered the fault.
        addr: *mut c_void,
        /// The thread that triggered the fault, if [`FeatureFlags::THREAD_ID`] is enabled.
        ///
        /// If the thread ID feature is not enabled, the value of this field is undefined. It would
        /// not be undefined behavior to use it, strictly speaking, but the [`Pid`] will not
        /// necessarily point to a real thread.
        ///
        /// This requires this crate to be compiled with the `linux4_14` feature.
        #[cfg(linux4_14)]
        thread_id: Pid,
    },
    /// Generated when the faulting process invokes `fork(2)` (or `clone(2)` without the `CLONE_VM`
    /// flag).
    Fork {
        /// The `Uffd` object created for the child by `fork(2)`
        uffd: Uffd,
    },
    /// Generated when the faulting process invokes `mremap(2)`.
    Remap {
        /// The original address of the memory range that was remapped.
        from: *mut c_void,
        /// The new address of the memory range that was remapped.
        to: *mut c_void,
        /// The original length of the memory range that was remapped.
        len: usize,
    },
    /// Generated when the faulting process invokes `madvise(2)` with `MADV_DONTNEED` or
    /// `MADV_REMOVE` advice.
    Remove {
        /// The start address of the memory range that was freed.
        start: *mut c_void,
        /// The end address of the memory range that was freed.
        end: *mut c_void,
    },
    /// Generated when the faulting process unmaps a meomry range, either explicitly using
    /// `munmap(2)` or implicitly during `mmap(2)` or `mremap(2)`.
    Unmap {
        /// The start address of the memory range that was unmapped.
        start: *mut c_void,
        /// The end address of the memory range that was unmapped.
        end: *mut c_void,
    },
}

impl Event {
    pub(crate) fn from_uffd_msg(msg: &raw::uffd_msg) -> Result<Event> {
        match msg.event {
            raw::UFFD_EVENT_PAGEFAULT => {
                let pagefault = unsafe { msg.arg.pagefault };
                cfg_if::cfg_if!(
                    if #[cfg(feature = "linux5_7")] {
                        let kind = if pagefault.flags & raw::UFFD_PAGEFAULT_FLAG_WP != 0 {
                            FaultKind::WriteProtected
                        } else {
                            FaultKind::Missing
                        };
                    } else {
                        let kind = FaultKind::Missing;
                    }
                );

                let rw = if pagefault.flags & raw::UFFD_PAGEFAULT_FLAG_WRITE == 0 {
                    ReadWrite::Read
                } else {
                    ReadWrite::Write
                };
                #[cfg(linux4_14)]
                let thread_id = Pid::from_raw(unsafe { pagefault.feat.ptid });
                Ok(Event::Pagefault {
                    kind,
                    rw,
                    addr: pagefault.address as *mut c_void,
                    #[cfg(linux4_14)]
                    thread_id,
                })
            }
            raw::UFFD_EVENT_FORK => {
                let fork = unsafe { msg.arg.fork };
                Ok(Event::Fork {
                    uffd: unsafe { Uffd::from_raw_fd(fork.ufd as RawFd) },
                })
            }
            raw::UFFD_EVENT_REMAP => {
                let remap = unsafe { msg.arg.remap };
                Ok(Event::Remap {
                    from: remap.from as *mut c_void,
                    to: remap.to as *mut c_void,
                    len: remap.len as usize,
                })
            }
            raw::UFFD_EVENT_REMOVE => {
                let remove = unsafe { msg.arg.remove };
                Ok(Event::Remove {
                    start: remove.start as *mut c_void,
                    end: remove.end as *mut c_void,
                })
            }
            raw::UFFD_EVENT_UNMAP => {
                let remove = unsafe { msg.arg.remove };
                Ok(Event::Unmap {
                    start: remove.start as *mut c_void,
                    end: remove.end as *mut c_void,
                })
            }
            _ => Err(Error::UnrecognizedEvent(msg.event)),
        }
    }
}
