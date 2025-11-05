use std::fmt;

use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Serialize};

const VERSION_KEY: &str = "snowglobe_version";
const VERSION: &str = "1";

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    VersionMismatch { expected: String, got: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "json: {error}"),
            Self::VersionMismatch { expected, got } => {
                write!(f, "version mismatch: expected '{expected}', got '{got}'")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub trait Message: Serialize + DeserializeOwned {
    fn serialize(&self) -> String {
        let mut json = serde_json::to_value(self).unwrap();
        let obj = json.as_object_mut().unwrap();
        obj.insert(VERSION_KEY.into(), VERSION.into());
        json.to_string()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        let mut json: serde_json::Value = serde_json::from_slice(bytes)?;
        let obj = json
            .as_object_mut()
            .ok_or_else(|| serde_json::Error::custom("expected an object"))?;

        let version = obj
            .remove(VERSION_KEY)
            .ok_or_else(|| serde_json::Error::missing_field(VERSION_KEY))?;
        if version != VERSION {
            return Err(Error::VersionMismatch {
                expected: VERSION.into(),
                got: version.to_string(),
            });
        }

        let x = serde_json::from_value(json)?;
        Ok(x)
    }
}

impl Message for Info {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub scenes: Vec<String>,
}
