#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(any(windows, target_os = "linux")))]
mod unsupported;

#[cfg(windows)]
pub(crate) use self::windows::{collect, collect_os_info};

#[cfg(target_os = "linux")]
pub(crate) use self::linux::{collect, collect_os_info};

#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) use self::unsupported::{collect, collect_os_info};
