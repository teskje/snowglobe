use libc::{c_int, c_void, pid_t, pthread_attr_t, pthread_t};

use super::patch;

// https://man7.org/linux/man-pages/man2/getpid.2.html
patch! {
    fn getpid() -> pid_t
    |_ctx| {
        12345
    }
}

// https://man7.org/linux/man-pages/man3/pthread_create.3.html
patch! {
    fn pthread_create(
        _native: *mut pthread_t,
        _attr: *const pthread_attr_t,
        _f: extern "C" fn(*mut c_void) -> *mut c_void,
        _value: *mut c_void,
    ) -> c_int
    |_ctx| {
        libc::EPERM
    }
}
