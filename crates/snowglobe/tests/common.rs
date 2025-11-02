#![allow(dead_code)]

use std::fmt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::sync::Mutex;

#[derive(Debug)]
pub struct SceneOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl fmt::Display for SceneOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.status)?;

        let stdout = self.stdout.trim();
        if !stdout.is_empty() {
            writeln!(f, "stdout: {}", stdout)?;
        }

        let stderr = self.stderr.trim();
        if !stderr.is_empty() {
            writeln!(f, "stderr: {}", stderr)?;
        }

        Ok(())
    }
}

pub fn run_test_scene(scene: &str) -> SceneOutput {
    let bin = build_test_scenes();

    let output = Command::new(bin)
        .args(["run", scene])
        .args(["--rng-seed", "0"])
        .args(["--start-time", "0"])
        .output()
        .unwrap();

    SceneOutput {
        status: output.status,
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn build_test_scenes() -> PathBuf {
    static NEED_BUILD: Mutex<bool> = Mutex::new(true);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crate_dir = root.join("tests/scenes");
    let bin_path = crate_dir.join("target/debug/test-scenes");

    let mut need_build = NEED_BUILD.lock().expect("poisoned");
    if *need_build {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(crate_dir)
            .status()
            .unwrap();
        assert!(status.success());

        *need_build = false;
    }

    bin_path
}
