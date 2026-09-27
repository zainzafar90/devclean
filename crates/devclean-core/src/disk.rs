use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use serde::Serialize;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub total: u64,
    pub free: u64,
    pub free_percent: u8,
}

impl DiskInfo {
    pub fn is_low(&self) -> bool {
        self.free_percent < 10
    }
}

/// Space available to the user on the volume holding `path`, as `df` reports it.
pub fn disk_info(path: &Path) -> Result<DiskInfo> {
    let c_path =
        CString::new(path.as_os_str().as_bytes()).map_err(|err| CoreError::io(path, std::io::Error::other(err)))?;
    // SAFETY: statvfs is plain data; zeroed is a valid initial value.
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is NUL-terminated and `stats` is valid for writes.
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stats) } != 0 {
        return Err(CoreError::io(path, std::io::Error::last_os_error()));
    }
    let block = stats.f_frsize as u64;
    let total = stats.f_blocks as u64 * block;
    let free = stats.f_bavail as u64 * block;
    let free_percent = (free * 100).checked_div(total).unwrap_or(0) as u8;
    Ok(DiskInfo {
        total,
        free,
        free_percent,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_home_volume() {
        let info = super::disk_info(std::path::Path::new("/")).unwrap();
        assert!(info.total > info.free);
        assert!(info.free_percent <= 100);
    }
}
