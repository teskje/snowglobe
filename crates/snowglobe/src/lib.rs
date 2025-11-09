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

#[cfg(feature = "cli")]
pub mod cli;

use std::thread;
use std::time::{Duration, UNIX_EPOCH};

pub use crate::error::{Error, Result};
pub use crate::sim::Sim;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub rng_seed: u64,
    pub start_time: Duration,
}

pub fn simulation<F>(cfg: Config, scene: F)
where
    F: FnOnce(Sim) + Send,
{
    // Run the simulation in a separate thread, to get as much isolation as possible.
    // Even if we run each scene in a separate process, setup work occurs outside of the simulation
    // and can modify thread-local state. For example, applying `LOG_FILTER` requires constructing
    // `HashMap`s, which disturbs the `HashMap` random state.
    //
    // Note that this doesn't help if setup code modifies process-global state, so we can't allow
    // that.
    let res = thread::scope(|scope| {
        thread::Builder::new()
            .name("simulation".into())
            .stack_size(8 << 20)
            .spawn_scoped(scope, || {
                context::enter_simulation(cfg.rng_seed, cfg.start_time);

                let sim = turmoil::Builder::new()
                    .enable_random_order()
                    .epoch(UNIX_EPOCH.checked_add(cfg.start_time).unwrap())
                    .tick_duration(Duration::from_millis(1))
                    .rng_seed(cfg.rng_seed)
                    .build();

                scene(sim.into());
            })
            .expect("creation succeeds")
            .join()
    });

    if res.is_err() {
        panic!("simulation panicked");
    }
}
