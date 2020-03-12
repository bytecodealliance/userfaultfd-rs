pub use semver::{SemVerError, Version};

mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Get the version specified in `<linux/version.h>`.
pub fn linux_headers_version() -> Version {
    let version = raw::LINUX_VERSION_CODE as u64;
    let major = version >> 16;
    let minor = version >> 8 & 0xFF;
    let patch = version & 0xFF;
    Version::new(major, minor, patch)
}

/// Get the version specified by `uname -r`.
///
/// This treats everything after the `major.minor.patch` triple as build metadata.
pub fn linux_kernel_version() -> Result<Version, SemVerError> {
    let uname = nix::sys::utsname::uname();
    let pre_ver = Version::parse(uname.release())?;
    Ok(Version {
        pre: vec![],
        build: pre_ver.pre.clone(),
        ..pre_ver
    })
}
