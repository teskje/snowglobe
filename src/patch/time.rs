use libc::{c_int, clockid_t, timespec};

use super::patch;

// https://man7.org/linux/man-pages/man3/clock_gettime.3.html
patch! {
    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int
    |ctx| {
        unsafe {
            (*tp).tv_sec = ctx.time.as_secs() as i64;
            (*tp).tv_nsec = ctx.time.subsec_nanos() as i64;
        }
        0
    }
}
