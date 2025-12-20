mod fuzz;
mod scene_bundle;
mod target;

use std::io::{BufRead as _, BufReader};
use std::num::NonZero;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::mpsc;
use std::{env, iter, process, thread};

use anyhow::bail;
use clap::Parser as _;

use crate::scene_bundle::SceneBundle;

/// Run snowglobe simulations defined in scene bundles
#[derive(clap::Parser)]
#[command(bin_name = "cargo snowglobe")]
#[command(version)]
struct Args {
    /// Package with the target to run
    #[arg(long, short)]
    package: Option<String>,
    #[command(flatten)]
    target: TargetArgs,
    /// Build artifacts in release mode, with optimizations
    #[arg(long, short)]
    release: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
#[group(multiple = false)]
struct TargetArgs {
    /// Name of the bin target to run
    #[arg(long)]
    bin: Option<String>,
    /// Name of the example target to run
    #[arg(long)]
    example: Option<String>,
}

impl TargetArgs {
    fn kind_and_name(&self) -> (target::Kind, Option<&str>) {
        use target::Kind::*;
        if let Some(name) = &self.bin {
            (Bin, Some(name))
        } else if let Some(name) = &self.example {
            (Example, Some(name))
        } else {
            (Bin, None)
        }
    }
}

#[derive(clap::Subcommand)]
enum Command {
    /// List all scenes
    List,
    /// Run a scene
    Run(RunArgs),
    /// Fuzz one or all scenes
    Fuzz(FuzzArgs),
    /// Check determinism of a scene run
    CheckDeterminism(RunArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// Name of the scene
    scene: String,
    /// RNG seed for the simulation
    #[arg(long)]
    rng_seed: Option<u64>,
    /// Start time of the simulation, in epoch ms
    #[arg(long)]
    start_time: Option<u64>,
}

#[derive(clap::Args)]
struct FuzzArgs {
    /// Name of the scene to fuzz (default: all scenes)
    #[arg(long, short)]
    scene: Option<String>,
    /// Number of runs (default: ∞)
    #[arg(long, short = 'n')]
    runs: Option<u64>,
    /// Number of parallel jobs (default: # of CPUs)
    #[arg(long, short)]
    jobs: Option<NonZero<usize>>,
}

fn main() -> anyhow::Result<()> {
    let mut args: Vec<_> = env::args().collect();

    // Strip second arg if invoked as `cargo snowglobe`.
    if args.get(1).is_some_and(|a| a == "snowglobe") {
        args.remove(1);
    }

    let args = Args::parse_from(args);

    let package_name = args.package.as_deref();
    let (kind, name) = args.target.kind_and_name();
    let target_spec = target::select(package_name, kind, name)?;
    eprintln!("target: {target_spec}");

    let bundle_path = build(&target_spec, args.release)?;
    let bundle = SceneBundle::new(bundle_path)?;

    match args.command {
        Command::List => cmd_list(&bundle),
        Command::Run(args) => cmd_run(&bundle, &args)?,
        Command::CheckDeterminism(args) => cmd_check_determinism(&bundle, &args)?,
        Command::Fuzz(args) => cmd_fuzz(&bundle, &args)?,
    }

    Ok(())
}

fn build(target: &target::Spec, release: bool) -> anyhow::Result<PathBuf> {
    use cargo_metadata::Message;

    let mut cmd = process::Command::new("cargo");
    cmd.args(["build", "--message-format", "json"]);
    if release {
        cmd.arg("--release");
    }

    cmd.args(["--package", &target.package.name]);
    match target.kind {
        target::Kind::Bin => cmd.args(["--bin", &target.name]),
        target::Kind::Example => cmd.args(["--example", &target.name]),
    };

    let mut proc = cmd.stdout(Stdio::piped()).spawn()?;

    let stdout = proc.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let messages = Message::parse_stream(reader);

    let mut path = None;
    for message in messages {
        match message? {
            Message::CompilerArtifact(artifact) => {
                if target.matches_artifact(&artifact) {
                    let exe = artifact.executable.expect("target is executable");
                    path = Some(exe);
                }
            }
            Message::CompilerMessage(msg) => eprintln!("{}", msg.message),
            Message::TextLine(line) => eprintln!("{line}"),
            _ => {}
        }
    }

    let status = proc.wait()?;
    if !status.success() {
        bail!("building scene bundle failed ({status})");
    }

    let path = path.expect("build was successful");
    Ok(path.into())
}

fn cmd_list(bundle: &SceneBundle) {
    for name in bundle.scenes() {
        println!("{name}");
    }
}

fn cmd_run(bundle: &SceneBundle, args: &RunArgs) -> anyhow::Result<()> {
    let rng_seed = args.rng_seed.unwrap_or_else(rand::random);

    let mut proc = bundle.run(&args.scene, rng_seed, args.start_time, None)?;
    let stdout = BufReader::new(proc.stdout.take().unwrap());
    let stderr = BufReader::new(proc.stderr.take().unwrap());

    let stdout_thread = thread::spawn(move || {
        for line in stdout.lines() {
            eprintln!("{}", line.unwrap());
        }
    });
    let stderr_thread = thread::spawn(move || {
        for line in stderr.lines() {
            eprintln!("{}", line.unwrap());
        }
    });

    let status = proc.wait()?;
    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    if !status.success() {
        bail!("running scene bundle failed ({status})");
    }

    Ok(())
}

fn cmd_fuzz(bundle: &SceneBundle, args: &FuzzArgs) -> anyhow::Result<()> {
    let scenes = match &args.scene {
        Some(name) => vec![name.clone()],
        None => bundle.scenes().map(ToString::to_string).collect(),
    };
    let jobs = args.jobs.unwrap_or_else(|| {
        let x = thread::available_parallelism();
        x.unwrap_or(NonZero::new(1).unwrap())
    });
    let runs = args.runs.unwrap_or(u64::MAX);

    eprintln!("Fuzzing {} scene(s) with {jobs} jobs", scenes.len());
    fuzz::fuzz(bundle, &scenes, jobs, runs)
}

fn cmd_check_determinism(bundle: &SceneBundle, args: &RunArgs) -> anyhow::Result<()> {
    let rng_seed = args.rng_seed.unwrap_or_else(rand::random);
    let log_filter = Some("trace");

    let mut proc1 = bundle.run(&args.scene, rng_seed, args.start_time, log_filter)?;
    let stdout1 = BufReader::new(proc1.stdout.take().unwrap());
    let stderr1 = BufReader::new(proc1.stderr.take().unwrap());

    let mut proc2 = bundle.run(&args.scene, rng_seed, args.start_time, log_filter)?;
    let stdout2 = BufReader::new(proc2.stdout.take().unwrap());
    let stderr2 = BufReader::new(proc2.stderr.take().unwrap());

    let (tx1, rx) = mpsc::channel();
    let tx2 = tx1.clone();

    let stdout_thread = thread::spawn(move || {
        let pairs = iter::zip(stdout1.lines(), stdout2.lines());
        for (line1, line2) in pairs {
            let (line1, line2) = (line1.unwrap(), line2.unwrap());
            if line1 != line2 {
                tx1.send((line1, line2)).unwrap();
            }
        }
    });
    let stderr_thread = thread::spawn(move || {
        let pairs = iter::zip(stderr1.lines(), stderr2.lines());
        for (line1, line2) in pairs {
            let (line1, line2) = (line1.unwrap(), line2.unwrap());
            if line1 != line2 {
                tx2.send((line1, line2)).unwrap();
            }
        }
    });

    let result = rx.recv();

    proc1.kill()?;
    proc2.kill()?;
    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    if let Ok(mismatch) = result {
        let (line1, line2) = mismatch;
        eprintln!("mismatch:\n\t1: {line1}\n\t2: {line2}");
        bail!("scene produced non-deterministic output");
    }

    eprintln!("determinism check successful");
    Ok(())
}
