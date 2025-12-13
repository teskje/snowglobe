use std::alloc::Layout;
use std::ptr::NonNull;
use std::{mem, ptr};

use libc::{EINVAL, ENOMEM, c_int, c_void, size_t};

use crate::alloc::ALLOCATOR;

use super::patch;

const ALIGN: usize = mem::size_of::<libc::max_align_t>();

// https://man7.org/linux/man-pages/man3/malloc.3.html
patch! {
    fn malloc(size: size_t) -> *mut c_void
    |_ctx| {
        let layout = Layout::from_size_align(size, ALIGN).unwrap();
        match ALLOCATOR.allocate(layout) {
            Some(ptr) => ptr.as_ptr().cast(),
            None => ptr::null_mut(),
        }
    }
}

// https://man7.org/linux/man-pages/man3/free.3.html
patch! {
    fn free(p: *mut c_void) -> ()
    |_ctx| {
        if let Some(ptr) = NonNull::new(p) {
            unsafe { ALLOCATOR.deallocate(ptr.cast(), ALIGN) };
        }
    }
}

// https://man7.org/linux/man-pages/man3/calloc.3.html
patch! {
    fn calloc(n: size_t, size: size_t) -> *mut c_void
    |_ctx| {
        let Some(size) = n.checked_mul(size) else {
            return ptr::null_mut();
        };

        let layout = Layout::from_size_align(size, ALIGN).unwrap();
        match ALLOCATOR.allocate(layout) {
            Some(ptr) => {
                unsafe { ptr.write_bytes(0, size) };
                ptr.as_ptr().cast()
            }
            None => ptr::null_mut(),
        }

    }
}

// https://man7.org/linux/man-pages/man3/realloc.3.html
patch! {
    fn realloc(p: *mut c_void, size: size_t) -> *mut c_void
    |_ctx| {
        let Some(ptr) = NonNull::new(p) else {
            return ptr::null_mut();
        };

        let new_layout = Layout::from_size_align(size, ALIGN).unwrap();
        match unsafe { ALLOCATOR.reallocate(ptr.cast(), new_layout) } {
            Some(ptr) => ptr.as_ptr().cast(),
            None => ptr::null_mut(),
        }
    }
}

// https://man7.org/linux/man-pages/man3/reallocarray.3.html
patch! {
    fn reallocarray(p: *mut c_void, n: size_t, size: size_t) -> *mut c_void
    |_ctx| {
        let Some(size) = n.checked_mul(size) else {
            return ptr::null_mut();
        };

        unsafe { realloc(p, size) }
    }
}

// https://man7.org/linux/man-pages/man3/posix_memalign.3.html
patch! {
    fn posix_memalign(memptr: *mut *mut c_void, alignment: size_t, size: size_t) -> c_int
    |_ctx| {
        if alignment % mem::size_of::<*mut c_void>() != 0 {
            return EINVAL;
        }
        let Ok(layout) = Layout::from_size_align(size, alignment) else {
            return EINVAL;
        };

        match ALLOCATOR.allocate(layout) {
            Some(ptr) => {
                unsafe { memptr.write(ptr.as_ptr().cast()) };
                0
            }
            None => ENOMEM,
        }
    }
}

// https://man7.org/linux/man-pages/man3/aligned_alloc.3.html
patch! {
    fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void
    |_ctx| {
        let Ok(layout) = Layout::from_size_align(size, alignment) else {
            return ptr::null_mut();
        };

        match ALLOCATOR.allocate(layout) {
            Some(ptr) => ptr.as_ptr().cast(),
            None => ptr::null_mut(),
        }
    }
}

// https://man7.org/linux/man-pages/man3/memalign.3.html
patch! {
    fn memalign(alignment: size_t, size: size_t) -> *mut c_void
    |_ctx| {
        let Ok(layout) = Layout::from_size_align(size, alignment) else {
            return ptr::null_mut();
        };

        match ALLOCATOR.allocate(layout) {
            Some(ptr) => ptr.as_ptr().cast(),
            None => ptr::null_mut(),
        }
    }
}

patch! {
    fn malloc_usable_size(_p: *mut c_void) -> size_t
    |_ctx| {
        unimplemented!("malloc_usable_size")
    }
}

patch! {
    fn valloc(_size: size_t) -> *mut c_void
    |_ctx| {
        unimplemented!("valloc")
    }
}

patch! {
    fn pvalloc(_size: size_t) -> *mut c_void
    |_ctx| {
        unimplemented!("pvalloc")
    }
}

patch! {
    fn reallocf(_p: *mut c_void, _size: size_t) -> *mut c_void
    |_ctx| {
        unimplemented!("reallocf")
    }
}

patch! {
    fn cfree(_p: *mut c_void) -> ()
    |_ctx| {
        unimplemented!("cfree")
    }
}
