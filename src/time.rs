use std::cell::Cell;
use std::time::Duration;

use libc::{c_int, clockid_t, timespec};

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

/// Patch a libc time function.
///
/// Inside a simulation, the libc function's behavior is overridden with the logic defined by the
/// provided body. Outside of simulation, the real libc function is invoked instead.
macro_rules! patch {
    (
        fn $name:ident($( $argname:ident : $argty:ty ),* $(,)?) -> $retty:ty
        | $time:ident | $body:block
    ) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($( $argname: $argty ),*) -> $retty {
            match TIME.get() {
                Some($time) => $body,
                None => {
                    let real = crate::dlsym! { $name($( $argty ),*) -> $retty };
                    unsafe { real($( $argname ),*) }
                }
            }
        }
    };
}

// https://man7.org/linux/man-pages/man3/clock_gettime.3.html
patch! {
    fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int
    |time| {
        unsafe {
            (*tp).tv_sec = time.as_secs() as i64;
            (*tp).tv_nsec = time.subsec_nanos() as i64;
        }
        0
    }
}
