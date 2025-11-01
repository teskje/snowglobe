use std::cell::Cell;
use std::mem;
use std::sync::OnceLock;
use std::time::Duration;

use libc::{RTLD_NEXT, c_int, clockid_t, timespec};

thread_local! {
    static TIME: Cell<Option<Duration>> = Cell::new(None);
}

pub(crate) fn enter_simulation(start_time: Duration) {
    TIME.set(Some(start_time));
}

pub(crate) fn exit_simulation() {
    TIME.set(None);
}

pub(crate) fn advance(new_time: Duration) {
    assert!(TIME.get().is_some_and(|t| t <= new_time));
    TIME.set(Some(new_time));
}

unsafe extern "C" fn real_clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
    static SYM: OnceLock<unsafe extern "C" fn(clockid_t, *mut timespec) -> c_int> = OnceLock::new();

    let f = SYM.get_or_init(|| unsafe {
        let ptr = libc::dlsym(RTLD_NEXT, c"clock_gettime".as_ptr().cast());
        assert!(!ptr.is_null());
        mem::transmute(ptr)
    });
    unsafe { f(clk_id, tp) }
}

/// https://man7.org/linux/man-pages/man3/clock_gettime.3.html
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
    let Some(time) = TIME.get() else {
        return unsafe { real_clock_gettime(clk_id, tp) };
    };

    unsafe {
        (*tp).tv_sec = time.as_secs() as i64;
        (*tp).tv_nsec = time.subsec_nanos() as i64;
    }
    0
}
