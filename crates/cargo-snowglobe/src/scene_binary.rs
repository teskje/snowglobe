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

        let status = cmd.status()?;
        if !status.success() {
            bail!("running scene binary failed ({status})");
        }

        Ok(())
    }
}
