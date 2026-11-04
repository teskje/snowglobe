use std::cell::RefCell;
use std::time::Duration;

use rand::SeedableRng;
use rand::rngs::SmallRng;

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

pub(crate) struct Context {
    rng_seed: u64,
    pub rng: SmallRng,
    pub time: Duration,
}

impl Context {
    fn new() -> Self {
        Self {
            rng_seed: 0,
            rng: SmallRng::seed_from_u64(0),
            time: Duration::ZERO,
        }
    }
}

pub(crate) fn with<F, R>(f: F) -> R
where
    F: FnOnce(&mut Context) -> R,
{
    CONTEXT.with_borrow_mut(f)
}

pub(crate) fn init_rng(seed: u64) {
    with(|ctx| {
        ctx.rng_seed = seed;
        ctx.rng = SmallRng::seed_from_u64(seed);
    });
}

/// The seed passed to --rng-seed for the current simulation.
pub fn rng_seed() -> u64 {
    with(|ctx| ctx.rng_seed)
}

pub(crate) fn advance_time(new_time: Duration) {
    with(|ctx| {
        assert!(ctx.time <= new_time);
        ctx.time = new_time;
    });
}
