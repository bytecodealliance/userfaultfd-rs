### Unreleased

- Added `Uffd::read_events` that can read multiple events from the userfaultfd file descriptor.
- Updated `bitflags` dependency to `2.2.1`.
- Use `/dev/userfaultfd` as the default API for creating userfaultfd file descriptors.

  Since Linux 5.11 a process can select if it wants to handle page faults triggered in kernel space
  or not. Under this mechanism, processes that wish to handle those, need to have `CAP_SYS_PTRACE`
  capability. `CAP_SYS_PTRACE` allows a process to do much more than create userfault fds, so with
  6.1 Linux introduces `/dev/userfaultfd`, a special character device that allows creating
  userfault file descriptors using the `USERFAULTFD_IOC_NEW` `ioctl`. Access to this device is
  granted via file system permissions and does not require `CAP_SYS_PTRACE` to handle kernel
  triggered page faults.

  We now default to using `/dev/userfaultfd` for creating the descriptors and only if that file is
  not present, we fall back to using the syscall.

### 0.3.1 (2021-02-17)

- Added support for the `UFFD_FEATURE_THREAD_ID` flag when compiled with the `linux4_14` Cargo
  feature.

### 0.3.0 (2021-02-03)

- Update `bindgen` dependency of `userfaultfd-sys` to `0.57`. Thank you @jgowans

### 0.2.1 (2020-11-20)

- Make `ReadWrite` public. Thank you @electroCutie

### 0.2.0 (2020-04-10)

- Removed the compile-time Linux version check, and replaced it with a Cargo feature.

  The Linux version check was overly restrictive, even on systems that did have the right kernel
  version installed but had older headers in `/usr/include/linux`. Beyond that, this check made it
  more difficult to compile on a different host than what's targeted.

  There is now a `linux4_14` feature flag on `userfaultfd-sys`, which turns on and tests the extra
  constants available in that version. Since `userfaultfd` did not make use of any of those newer
  features, it doesn't have a feature flag yet.

  Applications should take care when initializing with `UffdBuilder` to specify the features and
  ioctls they require, so that an unsupported version will be detected at runtime.


### 0.1.0 (2020-04-07)

- Initial public release of userfaultfd-rs.
