// Turmoil can only manually seed tokio runtimes under `tokio_unstable`, so we require this cfg to
// avoid accidental loss of determinism.
#[cfg(all(not(tokio_unstable), not(doc), not(doctest)))]
compile_error!("This crate requires `--cfg tokio_unstable`");

mod rng;
mod sim;
mod time;

use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use crate::sim::Sim;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub rng_seed: u64,
    pub start_time: Duration,
}

pub fn simulation<F>(cfg: Config, program: F)
where
    F: FnOnce(Sim) + Send + 'static,
{
    // Run the simulation in its own thread, the most reliable way to get rid of thread-local state
    // that was previously initialized. For example, we get a new `rand::ThreadRng` and don't have
    // to worry about reseeding the one in the current thread.
    thread::spawn(move || {
        rng::enter_simulation(cfg.rng_seed);
        time::enter_simulation(cfg.start_time);

        let sim = turmoil::Builder::new()
            .enable_random_order()
            .epoch(UNIX_EPOCH.checked_add(cfg.start_time).unwrap())
            .tick_duration(Duration::from_millis(1))
            .rng_seed(cfg.rng_seed)
            .build();

        program(sim.into());

        rng::exit_simulation();
        time::exit_simulation();
    })
    .join()
    .unwrap();
}

/// Dynamically load the given libc function.
///
/// Used during libc patching, to fall back to the real implementations when simulation is
/// disabled.
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
pub(crate) use dlsym;
