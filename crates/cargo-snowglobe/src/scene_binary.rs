use std::path::PathBuf;
use std::process;

use anyhow::{Context, bail};
use snowglobe_proto as proto;
use snowglobe_proto::Message as _;

pub struct SceneBinary {
    path: PathBuf,
    scenes: Vec<String>,
}

impl SceneBinary {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let output = process::Command::new(&path)
            .arg("info")
            .stderr(process::Stdio::inherit())
            .output()?;

        if !output.status.success() {
            bail!("running scene binary failed");
        }

        let info = proto::Info::deserialize(&output.stdout).context("parsing scene binary info")?;

        Ok(Self {
            path,
            scenes: info.scenes,
        })
    }

    pub fn scenes(&self) -> impl Iterator<Item = &str> {
        self.scenes.iter().map(|s| &s[..])
    }

    pub fn run(
        &self,
        scene: &str,
        rng_seed: Option<u64>,
        start_time: Option<u64>,
    ) -> anyhow::Result<()> {
        let seed = rng_seed.unwrap_or_else(rand::random);

        let mut cmd = process::Command::new(&self.path);
        cmd.args(["run", scene]);
        cmd.args(["--rng-seed", &seed.to_string()]);

        if let Some(time) = start_time {
            cmd.args(["--start-time", &time.to_string()]);
        }

        #[cfg(target_os = "linux")]
        disable_aslr(&mut cmd);

        let status = cmd.status()?;
        if !status.success() {
            bail!("running scene binary failed ({status})");
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn disable_aslr(cmd: &mut process::Command) {
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
