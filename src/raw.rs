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

// ioctls for /dev/userfaultfd

// This is the `/dev/userfaultfd` ioctl() from creating a new userfault file descriptor.
// It is a "bad" ioctl in the sense that it is defined as an _IOC:
// https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/userfaultfd.h#L17,
// aka `nix::ioctl_none`, however it does receive an integer argument:
// https://elixir.bootlin.com/linux/latest/source/fs/userfaultfd.c#L2186. That is the same argument
// that the userfaultfd() system call receives.
nix::ioctl_write_int_bad!(
    /// Create a new userfault file descriptor from the `/dev/userfaultfd`
    /// device. This receives the same arguments as the userfaultfd system call.
    new_uffd,
    nix::request_code_none!(USERFAULTFD_IOC, 0x00)
);
