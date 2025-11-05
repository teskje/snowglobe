use std::cell::RefCell;
use std::time::Duration;

use rand::SeedableRng;
use rand::rngs::SmallRng;

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

pub(crate) struct Context {
    pub rng: SmallRng,
    pub time: Duration,
}

pub(crate) fn with_or<FS, FR, R>(simulated: FS, real: FR) -> R
where
    FS: FnOnce(&mut Context) -> R,
    FR: FnOnce() -> R,
{
    CONTEXT.with_borrow_mut(|ctx| match ctx {
        Some(ctx) => simulated(ctx),
        None => real(),
    })
}

pub(crate) fn enter_simulation(seed: u64, start_time: Duration) {
    let ctx = Context {
        rng: SmallRng::seed_from_u64(seed),
        time: start_time,
    };
    CONTEXT.set(Some(ctx));
}

pub(crate) fn exit_simulation() {
    CONTEXT.set(None);
}

pub(crate) fn advance_time(new_time: Duration) {
    with_or(
        |ctx| {
            assert!(ctx.time <= new_time);
            ctx.time = new_time;
        },
        || panic!("cannot advance time outside simulation"),
    );
}
