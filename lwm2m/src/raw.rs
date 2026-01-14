// SPDX-FileCopyrightText: GARDENA GmbH
//
// SPDX-License-Identifier: MIT

//! Raw LWM2M types intended for clients.

use std::str::FromStr as _;

/// We use this so we can parse objects with unsupported resources.
#[derive(Debug, PartialEq, Hash, Eq)]
pub enum ObjectTypeMaybe {
    Parsed(crate::ObjectType),
    Unknown(String),
}

impl<'de> serde::Deserialize<'de> for ObjectTypeMaybe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match crate::ObjectType::from_str(&s) {
            Ok(t) => Self::Parsed(t),
            Err(_) => Self::Unknown(s),
        })
    }
}

pub type ObjectInstance = String;
pub type ResourceId = String;
pub type ResourceMap = std::collections::HashMap<ResourceId, crate::Value>;

/// An LWM2M object
#[derive(Debug, serde::Deserialize)]
pub struct Object {
    #[serde(flatten)]
    pub instances: std::collections::HashMap<ObjectInstance, ResourceMap>,
    #[serde(rename = "_urn")]
    pub urn: String,
}

/// the BNW id
pub type DeviceId = String;
pub type Device = std::collections::HashMap<ObjectTypeMaybe, Object>;
pub type DeviceMap = std::collections::HashMap<DeviceId, Device>;

#[cfg(test)]
mod tests {
    use anyhow::Context as _;

    fn device_json(path: &std::path::Path) -> Result<(), anyhow::Error> {
        let file = std::fs::File::open(path).context("can't open file")?;
        let reader = std::io::BufReader::new(file);
        let device: super::Device = serde_json::from_reader(reader).context("can't parse json")?;

        tracing::debug!("{:#?}", device);

        Ok(())
    }

    #[test_log::test]
    fn device_jsons() -> Result<(), anyhow::Error> {
        let dir =
            std::fs::read_dir("test/device_jsons").context("can't open device json directory")?;
        for entry in dir {
            let entry = entry.context("can't get device entry")?;

            device_json(&entry.path())
                .with_context(|| format!("failed to process entry `{entry:?}`"))
                .unwrap();
        }

        Ok(())
    }
}
