// Turmoil can only manually seed tokio runtimes under `tokio_unstable`, so we require this cfg to
// avoid accidental loss of determinism.
#[cfg(all(not(tokio_unstable), not(doc), not(doctest)))]
compile_error!("This crate requires `--cfg tokio_unstable`");

// By default on Linux, `rand` makes direct `getrandom` syscalls, which we can't patch. Using the
// "linux_getrandom" backend makes it call into libc instead.
#[cfg(all(
    target_os = "linux",
    not(getrandom_backend = "linux_getrandom"),
    not(doc),
    not(doctest),
))]
compile_error!("This crate requires `--cfg getrandom_backend=\"linux_getrandom\"");

mod context;
mod error;
mod patch;
mod sim;

use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use crate::sim::Sim;

pub use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub rng_seed: u64,
    pub start_time: Duration,
}

pub fn simulation<F>(cfg: Config, program: F) -> Result
where
    F: FnOnce(Sim) + Send + 'static,
{
    // Run the simulation in its own thread, the most reliable way to get rid of thread-local state
    // that was previously initialized. For example, we get a new `rand::ThreadRng` and don't have
    // to worry about reseeding the one in the current thread.
    //
    // What's more, running the simulation in a separate thread lets us not worry about having to
    // clean up the thread-local simulation context at the end. Once the thread finishes we end up
    // in a pristine state, even if the code inside the simulation panicked.
    thread::spawn(move || {
        context::enter_simulation(cfg.rng_seed, cfg.start_time);

        let sim = turmoil::Builder::new()
            .enable_random_order()
            .epoch(UNIX_EPOCH.checked_add(cfg.start_time).unwrap())
            .tick_duration(Duration::from_millis(1))
            .rng_seed(cfg.rng_seed)
            .build();

        program(sim.into());
    })
    .join()
    .map_err(error::downcast)
}
