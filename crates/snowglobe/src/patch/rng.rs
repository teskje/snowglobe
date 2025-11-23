use libc::{c_int, c_uint, c_void, size_t, ssize_t};
use rand::Rng;
use rand::rngs::SmallRng;

use super::patch;

unsafe fn fill_raw(rng: &mut SmallRng, buf: *mut u8, len: usize) {
    for i in 0..len {
        let b = rng.random();
        unsafe { buf.add(i).write(b) };
    }
}

// https://man7.org/linux/man-pages/man2/getrandom.2.html
patch! {
    fn getrandom(buf: *mut c_void, buflen: size_t, _flags: c_uint) -> ssize_t
    |ctx| {
        unsafe { fill_raw(&mut ctx.rng, buf.cast(), buflen) };
        buflen as ssize_t
    }
}

// https://man7.org/linux/man-pages/man3/getentropy.3.html
patch! {
    fn getentropy(buf: *mut c_void, buflen: size_t) -> c_int
    |ctx| {
        if buflen > 256 {
            return -1;
        }

        unsafe { fill_raw(&mut ctx.rng, buf.cast(), buflen) };
        0
    }
}

#[cfg(target_os = "macos")]
patch! {
    fn CCRandomGenerateBytes(bytes: *mut c_void, size: size_t) -> libc::CCRNGStatus
    |ctx| {
        unsafe { fill_raw(&mut ctx.rng, bytes.cast(), size) };
        libc::kCCSuccess
    }
}
