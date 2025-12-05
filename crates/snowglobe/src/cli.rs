use std::collections::BTreeMap;
use std::time::{Duration, UNIX_EPOCH};

use crate::{Result, Sim, context};

pub use snowglobe_macros::scene;
use snowglobe_proto as proto;
use snowglobe_proto::Message as _;
use tracing::info;

#[derive(argh::FromArgs)]
/// Run snowglobe simulations
struct Args {
    #[argh(subcommand)]
    command: Command,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Command {
    Info(InfoArgs),
    Run(RunArgs),
}

#[derive(argh::FromArgs)]
/// print information about this scene binary
#[argh(subcommand, name = "info")]
struct InfoArgs {}

#[derive(argh::FromArgs)]
/// run a scene
#[argh(subcommand, name = "run")]
struct RunArgs {
    /// name of the scene
    #[argh(positional)]
    scene: String,
    /// RNG seed for the simulation
    #[argh(option)]
    rng_seed: u64,
    /// start time of the simulation, in epoch ms
    #[argh(option)]
    start_time: Option<u64>,
}

pub fn main() -> Result {
    let args: Args = argh::from_env();
    init_logging();

    match args.command {
        Command::Info(_args) => info(),
        Command::Run(args) => run(args)?,
    }

    Ok(())
}

fn init_logging() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .init();
}

fn info() {
    let scenes = scenes().keys().cloned().collect();
    let info = proto::Info { scenes };

    print!("{}", info.serialize());
}

fn run(args: RunArgs) -> Result {
    let scenes = scenes();
    let scene = scenes.get(&args.scene).ok_or("scene does not exist")?;

    let rng_seed = args.rng_seed;
    let start_time_ms = args.start_time.unwrap_or(0);
    let start_time = Duration::from_millis(start_time_ms);

    info!(%rng_seed, ?start_time, "running simulation");

    context::init(rng_seed, start_time);

    let epoch = UNIX_EPOCH.checked_add(start_time).unwrap();
    let sim = turmoil::Builder::new()
        .enable_random_order()
        .epoch(epoch)
        .tick_duration(Duration::from_millis(1))
        .rng_seed(rng_seed)
        .build();

    scene(sim.into());

    Ok(())
}

fn scenes() -> BTreeMap<String, fn(Sim)> {
    __private::SCENES
        .iter()
        .map(|s| {
            let name = match s.module.split_once("::") {
                Some((_, path)) => format!("{path}::{}", s.name),
                None => s.name.into(),
            };
            (name, s.func)
        })
        .collect()
}

/// Internals used by macros.
#[doc(hidden)]
pub mod __private {
    pub use linkme;

    #[linkme::distributed_slice]
    pub static SCENES: [Scene];

    pub struct Scene {
        pub module: &'static str,
        pub name: &'static str,
        pub func: fn(crate::Sim),
    }
}
