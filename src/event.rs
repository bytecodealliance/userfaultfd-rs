use crate::error::{Error, Result};
use crate::raw;
use crate::Uffd;
use libc::c_void;
use std::os::unix::io::{FromRawFd, RawFd};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadWrite {
    Read,
    Write,
}

/// Events from the userfaultfd object that are read by `Uffd::read_event()`.
#[derive(Debug)]
pub enum Event {
    /// A pagefault event.
    Pagefault {
        /// Whether the fault is on a read or a write.
        rw: ReadWrite,
        /// The address that triggered the fault.
        addr: *mut c_void,
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
                let rw = if pagefault.flags & raw::UFFD_PAGEFAULT_FLAG_WRITE == 0 {
                    ReadWrite::Read
                } else {
                    ReadWrite::Write
                };
                Ok(Event::Pagefault {
                    rw,
                    addr: pagefault.address as *mut c_void,
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
