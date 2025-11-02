mod containment;
mod determinism;

use std::time::Duration;

struct Args {
    scene: String,
    rng_seed: Option<u64>,
    start_time: Option<Duration>,
}

impl Args {
    fn from_env() -> Self {
        let mut pargs = pico_args::Arguments::from_env();
        let scene = pargs.free_from_str().unwrap();
        let rng_seed = pargs.opt_value_from_str("--rng-seed").unwrap();
        let start_time_ms = pargs.opt_value_from_str("--start-time").unwrap();
        let start_time = start_time_ms.map(Duration::from_millis);

        Self {
            scene,
            rng_seed,
            start_time,
        }
    }
}

fn main() {
    let args = Args::from_env();
    let rng_seed = args.rng_seed.unwrap_or_else(|| rand::random());
    let start_time = args.start_time.unwrap_or(Duration::ZERO);

    let cfg = snowglobe::Config {
        rng_seed,
        start_time,
    };
    let sim = snowglobe::simulation(cfg);

    match &*args.scene {
        "determinism::random_numbers" => determinism::random_numbers(sim),
        "determinism::select_branch" => determinism::select_branch(sim),
        "determinism::hashset_order" => determinism::hashset_order(sim),
        "determinism::tokio_time" => determinism::tokio_time(sim),
        "determinism::std_time" => determinism::std_time(sim),
        "determinism::uuid" => determinism::uuid(sim),
        "containment::thread_spawn" => containment::thread_spawn(sim),
        "containment::tokio_spawn_blocking" => containment::tokio_spawn_blocking(sim),
        name => panic!("unknown scene: {name}"),
    }
}
