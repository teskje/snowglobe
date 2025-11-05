use std::collections::BTreeMap;
use std::time::Duration;

use crate::{Config, Result, Sim, simulation};

pub use snowglobe_macros::scene;
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
    List(ListArgs),
    Run(RunArgs),
}

#[derive(argh::FromArgs)]
/// list all scenes
#[argh(subcommand, name = "list")]
struct ListArgs {}

#[derive(argh::FromArgs)]
/// run a scene
#[argh(subcommand, name = "run")]
struct RunArgs {
    /// name of the scene
    #[argh(positional)]
    scene: String,
    /// RNG seed for the simulation
    #[argh(option)]
    rng_seed: Option<u64>,
    /// start time of the simulation, in epoch ms
    #[argh(option)]
    start_time: Option<u64>,
}

pub fn main() -> Result {
    let args: Args = argh::from_env();

    match args.command {
        Command::List(_args) => list(),
        Command::Run(args) => run(args)?,
    }

    Ok(())
}

fn list() {
    for name in scenes().keys() {
        println!("{name}");
    }
}

fn run(args: RunArgs) -> Result {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let scenes = scenes();
    let scene = scenes.get(&args.scene).ok_or("scene does not exist")?;

    let rng_seed = args.rng_seed.unwrap_or_else(rand::random);
    let start_time_ms = args.start_time.unwrap_or(0);

    let cfg = Config {
        rng_seed,
        start_time: Duration::from_millis(start_time_ms),
    };

    info!(?cfg, "running simulation");
    simulation(cfg, scene);

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
