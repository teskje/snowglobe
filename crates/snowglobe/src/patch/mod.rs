mod rng;
mod thread;
mod time;

/// Patch a libc function.
macro_rules! patch {
    (
        fn $name:ident($( $argname:ident : $argty:ty ),* $(,)?) -> $ret:ty
        | $ctx:ident | $logic:block
    ) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($( $argname: $argty ),*) -> $ret {
            crate::context::with(|$ctx| $logic)
        }
    };
}

use patch;
