use std::cell::RefCell;
use std::ffi::c_void;

use libc::{c_int, c_uint, size_t, ssize_t};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

thread_local! {
    static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);
}

pub(crate) fn enter_simulation(seed: u64) {
    RNG.set(Some(SmallRng::seed_from_u64(seed)));
}

pub(crate) fn exit_simulation() {
    RNG.set(None);
}

unsafe fn fill_raw(rng: &mut SmallRng, buf: *mut u8, len: usize) {
    for i in 0..len {
        let b = rng.random();
        unsafe { buf.add(i).write(b) };
    }
}

/// Patch a libc rng function.
///
/// Inside a simulation, the libc function's behavior is overridden with the logic defined by the
/// provided body. Outside of simulation, the real libc function is invoked instead.
macro_rules! patch {
    (
        fn $name:ident($( $argname:ident : $argty:ty ),* $(,)?) -> $retty:ty
        | $rng:ident | $body:block
    ) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($( $argname: $argty ),*) -> $retty {
            RNG.with_borrow_mut(|rng| match rng {
                Some($rng) => $body,
                None => {
                    let real = crate::dlsym! { $name($( $argty ),*) -> $retty };
                    unsafe { real($( $argname ),*) }
                }
            })
        }
    };
}

// https://man7.org/linux/man-pages/man2/getrandom.2.html
patch! {
    fn getrandom(buf: *mut c_void, buflen: size_t, flags: c_uint) -> ssize_t
    |rng| {
        unsafe { fill_raw(rng, buf.cast(), buflen) };
        buflen as ssize_t
    }
}

// https://man7.org/linux/man-pages/man3/getentropy.3.html
patch! {
    fn getentropy(buf: *mut c_void, buflen: size_t) -> c_int
    |rng| {
        if buflen > 256 {
            return -1;
        }

        unsafe { fill_raw(rng, buf.cast(), buflen) };
        0
    }
}

#[cfg(target_os = "macos")]
patch! {
    fn CCRandomGenerateBytes(bytes: *mut c_void, size: size_t) -> libc::CCRNGStatus
    |rng| {
        unsafe { fill_raw(rng, bytes.cast(), size) };
        libc::kCCSuccess
    }
}
