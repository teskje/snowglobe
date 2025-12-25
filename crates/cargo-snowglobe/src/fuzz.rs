use std::num::NonZero;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use std::{process, thread};

use anyhow::bail;
use rand::Rng;
use rand::seq::IndexedRandom;

use crate::scene_bundle::SceneBundle;

pub fn fuzz(
    bundle: &SceneBundle,
    scenes: &[String],
    jobs: NonZero<usize>,
    runs: u64,
) -> anyhow::Result<()> {
    let work = Work::new(runs);
    let (tx, rx) = mpsc::channel();

    for i in 0..jobs.get() {
        let bundle = bundle.clone();
        let scenes = scenes.to_vec();
        let work = work.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            if let Err(error) = run_job(&bundle, &scenes, work, tx) {
                eprintln!("error in job {i}: {error}; aborting");
                process::abort();
            }
        });
    }

    drop(tx);

    while let Ok(result) = rx.recv() {
        let RunResult {
            seed,
            scene,
            duration,
            output,
        } = result;

        if output.status.success() {
            eprintln!("ran scene {scene} in {duration:?}");
        } else {
            eprintln!("scene {scene} failed with status {}", output.status);
            eprintln!("seed: {seed}");
            eprintln!();
            eprintln!("--- stdout ---");
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            eprintln!();
            eprintln!("--- stderr ---");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            eprintln!();
            bail!("fuzzing encountered a failed run");
        }
    }

    Ok(())
}

struct RunResult {
    seed: u64,
    scene: String,
    duration: Duration,
    output: process::Output,
}

#[derive(Clone)]
struct Work(Arc<AtomicU64>);

impl Work {
    fn new(n: u64) -> Self {
        Self(Arc::new(AtomicU64::new(n)))
    }

    fn take(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let result = self.0.fetch_update(Relaxed, Relaxed, |n| n.checked_sub(1));
        result.is_ok()
    }
}

fn run_job(
    bundle: &SceneBundle,
    scenes: &[String],
    work: Work,
    tx: mpsc::Sender<RunResult>,
) -> anyhow::Result<()> {
    let mut rng = rand::rng();

    while work.take() {
        let scene = scenes.choose(&mut rng).unwrap();
        let seed = rng.random();

        let start = Instant::now();
        let proc = bundle.run(scene, seed, None)?;
        let output = proc.wait_with_output()?;
        let duration = start.elapsed();

        let result = RunResult {
            seed,
            scene: scene.to_string(),
            duration,
            output,
        };
        if tx.send(result).is_err() {
            break;
        }
    }

    Ok(())
}
