### Unreleased

- Added `Uffd::read_events` that can read multiple events from the userfaultfd file descriptor.

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
