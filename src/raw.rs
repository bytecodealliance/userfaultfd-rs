use libc::{c_int, c_long, syscall, SYS_userfaultfd, INT_MAX};
pub use userfaultfd_sys::*;

pub unsafe fn userfaultfd(flags: c_int) -> c_int {
    let fd = syscall(SYS_userfaultfd, flags as c_long);
    if fd > INT_MAX as c_long {
        panic!("fd doesn't fit in a c_int");
    } else {
        fd as c_int
    }
}

nix::ioctl_readwrite!(api, UFFDIO, _UFFDIO_API, uffdio_api);
nix::ioctl_readwrite!(register, UFFDIO, _UFFDIO_REGISTER, uffdio_register);
nix::ioctl_read!(unregister, UFFDIO, _UFFDIO_UNREGISTER, uffdio_range);
nix::ioctl_read!(wake, UFFDIO, _UFFDIO_WAKE, uffdio_range);
nix::ioctl_readwrite!(copy, UFFDIO, _UFFDIO_COPY, uffdio_copy);
nix::ioctl_readwrite!(zeropage, UFFDIO, _UFFDIO_ZEROPAGE, uffdio_zeropage);
#[cfg(feature = "linux5_7")]
nix::ioctl_readwrite!(
    write_protect,
    UFFDIO,
    _UFFDIO_WRITEPROTECT,
    uffdio_writeprotect
);
