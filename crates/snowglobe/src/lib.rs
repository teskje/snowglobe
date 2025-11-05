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
    F: FnOnce(Sim),
{
    context::enter_simulation(cfg.rng_seed, cfg.start_time);

    let sim = turmoil::Builder::new()
        .enable_random_order()
        .epoch(UNIX_EPOCH.checked_add(cfg.start_time).unwrap())
        .tick_duration(Duration::from_millis(1))
        .rng_seed(cfg.rng_seed)
        .build();

    scene(sim.into());

    context::exit_simulation();
}
