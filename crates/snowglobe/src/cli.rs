use std::collections::BTreeMap;
use std::time::Duration;

use crate::{Result, context};

use __private::*;
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
/// print information about this scene bundle
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
    info!(scene = args.scene, rng_seed, "running simulation");

    context::init_rng(rng_seed);
    run_scene(scene, rng_seed);

    Ok(())
}

fn run_scene(scene: &Scene, rng_seed: u64) {
    let mut builder = turmoil::Builder::new();
    builder.enable_random_order();
    builder.tick_duration(Duration::from_millis(1));
    builder.rng_seed(rng_seed);

    macro_rules! apply_config {
        ($cfg:expr, $builder:expr, [$( $arg:ident, )*]) => {
            $( if let Some(x) = $cfg.$arg { $builder.$arg(x); } )*
        };
    }

    apply_config!(
        scene.config,
        builder,
        [
            simulation_duration,
            tick_duration,
            min_message_latency,
            max_message_latency,
            fail_rate,
            repair_rate,
        ]
    );

    let sim = builder.build().into();
    (scene.func)(sim);
}

fn scenes() -> BTreeMap<String, &'static Scene> {
    SCENES
        .iter()
        .map(|s| {
            let name = match s.module.split_once("::") {
                Some((_, path)) => format!("{path}::{}", s.name),
                None => s.name.into(),
            };
            (name, s)
        })
        .collect()
}

/// Internals used by macros.
#[doc(hidden)]
pub mod __private {
    use std::time::Duration;

    pub use linkme;

    #[linkme::distributed_slice]
    pub static SCENES: [Scene];

    pub struct Scene {
        pub module: &'static str,
        pub name: &'static str,
        pub func: fn(crate::Sim),
        pub config: SceneConfig,
    }

    #[derive(Debug, PartialEq)]
    pub struct SceneConfig {
        pub simulation_duration: Option<Duration>,
        pub tick_duration: Option<Duration>,
        pub min_message_latency: Option<Duration>,
        pub max_message_latency: Option<Duration>,
        pub fail_rate: Option<f64>,
        pub repair_rate: Option<f64>,
    }
}
