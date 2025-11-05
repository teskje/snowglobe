//! Target selection.

use std::fmt::{self, Write as _};

use cargo_metadata::{Artifact, MetadataCommand, PackageId};

pub struct Spec {
    pub package: Package,
    pub kind: Kind,
    pub name: String,
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} `{}` in package `{}`",
            self.kind, self.name, self.package
        )
    }
}

impl Spec {
    pub fn matches_artifact(&self, artifact: &Artifact) -> bool {
        self.package.id == artifact.package_id
            && artifact.target.is_kind(self.kind.into())
            && self.name == artifact.target.name
    }
}

pub struct Package {
    pub name: String,
    pub id: PackageId,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Bin,
    Example,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bin => f.write_str("bin"),
            Self::Example => f.write_str("example"),
        }
    }
}

impl From<Kind> for cargo_metadata::TargetKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Bin => Self::Bin,
            Kind::Example => Self::Example,
        }
    }
}

pub fn select(package_name: Option<&str>, kind: Kind, name: Option<&str>) -> anyhow::Result<Spec> {
    let metadata = MetadataCommand::new().no_deps().exec()?;

    let mut candidates = Vec::new();
    for package in &metadata.packages {
        if package_name.is_some_and(|p| package.name != p) {
            continue;
        }

        for target in &package.targets {
            if name.is_some_and(|n| target.name != *n) {
                continue;
            }

            if (target.is_bin() && kind == Kind::Bin)
                || (target.is_example() && kind == Kind::Example)
            {
                candidates.push(Spec {
                    package: Package {
                        name: package.name.to_string(),
                        id: package.id.clone(),
                    },
                    kind,
                    name: target.name.to_string(),
                });
            }
        }
    }

    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => Err(error_no_target(kind, name)),
        _ => Err(error_multiple_targets(kind, name, &candidates)),
    }
}

fn error_no_target(kind: Kind, name: Option<&str>) -> anyhow::Error {
    let mut msg = String::new();
    let m = &mut msg;

    write!(m, "no {kind} target").unwrap();
    if let Some(name) = name {
        write!(m, " named `{name}`").unwrap();
    }
    write!(m, " found").unwrap();

    anyhow::Error::msg(msg)
}

fn error_multiple_targets(kind: Kind, name: Option<&str>, candidates: &[Spec]) -> anyhow::Error {
    let mut msg = String::new();
    let m = &mut msg;

    write!(m, "multiple {kind} targets").unwrap();
    if let Some(name) = name {
        write!(m, " named `{name}`").unwrap();
    }
    write!(m, " found:").unwrap();
    for spec in candidates {
        write!(m, "\n    {spec}").unwrap();
    }

    anyhow::Error::msg(msg)
}
