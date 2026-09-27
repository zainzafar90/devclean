use std::ffi::CStr;

/// Reads an integer sysctl by name; `None` when the key does not exist on this macOS.
pub fn read_i32(name: &CStr) -> Option<i32> {
    let mut value: libc::c_int = 0;
    let mut size = std::mem::size_of::<libc::c_int>();
    // SAFETY: `value` and `size` are valid for writes and describe a c_int buffer.
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&mut value as *mut libc::c_int).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    (rc == 0).then_some(value)
}
