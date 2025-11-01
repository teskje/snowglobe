mod rng;
mod time;

/// Dynamically load the given libc function.
macro_rules! dlsym {
    ( $name:ident($( $arg:ty ),*) -> $ret:ty ) => {{
        use std::ffi::CString;
        use std::sync::OnceLock;

        static SYM: OnceLock<unsafe extern "C" fn($( $arg ),*) -> $ret> = OnceLock::new();

        SYM.get_or_init(|| unsafe {
            let name = CString::new(stringify!($name)).unwrap();
            let ptr = libc::dlsym(libc::RTLD_NEXT, name.as_ptr().cast());
            assert!(!ptr.is_null());
            std::mem::transmute(ptr)
        })
    }};
}

/// Patch a libc function.
///
/// Inside a simulation, the libc function's behavior is overridden with the provided logic.
/// Outside of simulation, the real libc function is invoked instead.
macro_rules! patch {
    (
        fn $name:ident($( $argname:ident : $argty:ty ),* $(,)?) -> $ret:ty
        | $rng:ident | $logic:block
    ) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($( $argname: $argty ),*) -> $ret {
            crate::context::with_or(
                |$rng| $logic,
                || {
                    let real = crate::patch::dlsym! { $name($( $argty ),*) -> $ret };
                    unsafe { real($( $argname ),*) }
                }
            )
        }
    };
}

use dlsym;
use patch;
