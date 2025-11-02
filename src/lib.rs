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

use std::time::{Duration, UNIX_EPOCH};

use rand::SeedableRng;
use rand::rngs::SmallRng;

pub use crate::error::{Error, Result};
pub use crate::sim::Sim;

pub fn seed(rng_seed: u64, start_time: Duration) {
    context::with(|ctx| {
        ctx.rng = SmallRng::seed_from_u64(rng_seed);
        ctx.time = start_time;
    })
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub rng_seed: u64,
    pub start_time: Duration,
}

pub fn simulation(cfg: Config) -> Sim {
    context::seed(cfg.rng_seed, cfg.start_time);

    turmoil::Builder::new()
        .enable_random_order()
        .epoch(UNIX_EPOCH.checked_add(cfg.start_time).unwrap())
        .tick_duration(Duration::from_millis(1))
        .rng_seed(cfg.rng_seed)
        .build()
        .into()
}
