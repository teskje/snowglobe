#![allow(dead_code)]

use std::fmt;
use std::process::{Command, ExitStatus};

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
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--example", "test-scenes"])
        .arg("--")
        .args(["run", scene])
        .args(["--rng-seed", "0"])
        .args(["--start-time", "0"]);

    #[cfg(target_os = "linux")]
    disable_aslr(&mut cmd);

    let output = cmd.output().unwrap();

    SceneOutput {
        status: output.status,
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

#[cfg(target_os = "linux")]
fn disable_aslr(cmd: &mut Command) {
    use libc::{ADDR_NO_RANDOMIZE, c_ulong, personality};
    use std::os::unix::process::CommandExt;

    unsafe {
        cmd.pre_exec(|| {
            let old = personality(0xffffffff);
            assert_ne!(old, -1);
            let new = (old | ADDR_NO_RANDOMIZE) as c_ulong;
            assert_ne!(personality(new), -1);
            Ok(())
        });
    }
}
