use std::cell::RefCell;
use std::ffi::c_void;
use std::mem;
use std::sync::OnceLock;

use libc::{RTLD_NEXT, c_int, c_uint, size_t, ssize_t};
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

/// https://man7.org/linux/man-pages/man2/getrandom.2.html
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, flags: c_uint) -> ssize_t {
    RNG.with_borrow_mut(|rng| {
        let Some(rng) = rng else {
            return unsafe { real_getrandom(buf, buflen, flags) };
        };

        unsafe { fill_raw(rng, buf.cast(), buflen) };
        buflen as ssize_t
    })
}

unsafe extern "C" fn real_getrandom(buf: *mut c_void, buflen: size_t, flags: c_uint) -> ssize_t {
    static SYM: OnceLock<unsafe extern "C" fn(*mut c_void, size_t, c_uint) -> ssize_t> =
        OnceLock::new();

    let f = SYM.get_or_init(|| unsafe {
        let ptr = libc::dlsym(RTLD_NEXT, c"getrandom".as_ptr().cast());
        assert!(!ptr.is_null());
        mem::transmute(ptr)
    });
    unsafe { f(buf, buflen, flags) }
}

/// https://man7.org/linux/man-pages/man3/getentropy.3.html
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getentropy(buf: *mut c_void, buflen: size_t) -> c_int {
    RNG.with_borrow_mut(|rng| {
        let Some(rng) = rng else {
            return unsafe { real_getentropy(buf, buflen) };
        };

        if buflen <= 256 {
            unsafe { fill_raw(rng, buf.cast(), buflen) };
            0
        } else {
            -1
        }
    })
}

unsafe extern "C" fn real_getentropy(buf: *mut c_void, buflen: size_t) -> c_int {
    static SYM: OnceLock<unsafe extern "C" fn(*mut c_void, size_t) -> c_int> = OnceLock::new();

    let f = SYM.get_or_init(|| unsafe {
        let ptr = libc::dlsym(RTLD_NEXT, c"getentropy".as_ptr().cast());
        assert!(!ptr.is_null());
        mem::transmute(ptr)
    });
    unsafe { f(buf, buflen) }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CCRandomGenerateBytes(
    bytes: *mut c_void,
    size: size_t,
) -> libc::CCRNGStatus {
    RNG.with_borrow_mut(|rng| {
        let Some(rng) = rng else {
            return unsafe { real_CCRandomGenerateBytes(bytes, size) };
        };

        unsafe { fill_raw(rng, bytes.cast(), size) };
        libc::kCCSuccess
    })
}

#[cfg(target_os = "macos")]
#[allow(non_snake_case)]
unsafe extern "C" fn real_CCRandomGenerateBytes(
    bytes: *mut c_void,
    size: size_t,
) -> libc::CCRNGStatus {
    static SYM: OnceLock<unsafe extern "C" fn(*mut c_void, size_t) -> c_int> = OnceLock::new();

    let f = SYM.get_or_init(|| unsafe {
        let ptr = libc::dlsym(RTLD_NEXT, c"CCRandomGenerateBytes".as_ptr().cast());
        assert!(!ptr.is_null());
        mem::transmute(ptr)
    });
    unsafe { f(bytes, size) }
}
