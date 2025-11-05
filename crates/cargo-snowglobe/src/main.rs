mod scene_binary;
mod target;

use std::io::BufReader;
use std::path::PathBuf;
use std::process::Stdio;
use std::{env, process};

use anyhow::bail;
use clap::Parser as _;

use crate::scene_binary::SceneBinary;

/// Run snowglobe simulations defined in scene bundles
#[derive(clap::Parser)]
#[command(bin_name = "cargo snowglobe")]
#[command(version)]
struct Args {
    #[arg(short, long)]
    package: Option<String>,
    #[command(flatten)]
    target: TargetArgs,

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
    println!("target: {target_spec}");

    let binary_path = build(&target_spec)?;
    let binary = SceneBinary::new(binary_path)?;

    match args.command {
        Command::List => list(&binary),
        Command::Run(args) => run(&binary, &args)?,
    }

    Ok(())
}

fn build(target: &target::Spec) -> anyhow::Result<PathBuf> {
    use cargo_metadata::Message;

    let mut cmd = process::Command::new("cargo");
    cmd.args(["build", "--message-format", "json"]);

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
            Message::CompilerMessage(msg) => println!("{}", msg.message),
            Message::TextLine(line) => println!("{line}"),
            _ => {}
        }
    }

    let status = proc.wait()?;
    if !status.success() {
        bail!("building scene binary failed ({status})");
    }

    let path = path.expect("build was successful");
    Ok(path.into())
}

fn list(binary: &SceneBinary) {
    for name in binary.scenes() {
        println!("{name}");
    }
}

fn run(binary: &SceneBinary, args: &RunArgs) -> anyhow::Result<()> {
    binary.run(&args.scene, args.rng_seed, args.start_time)
}
