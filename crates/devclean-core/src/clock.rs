/// Local time as `YYYY-MM-DD HH:MM:SS`, the log timestamp format.
pub fn local_timestamp() -> String {
    // SAFETY: time(NULL) and localtime_r with valid out-pointers are thread-safe libc calls.
    let tm = unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        tm
    };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn has_log_shape() {
        let stamp = super::local_timestamp();
        assert_eq!(stamp.len(), 19);
        assert!(stamp.starts_with("20"));
    }
}
