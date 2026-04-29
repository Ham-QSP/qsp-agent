/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub name: String,
    pub description: String,
    pub signaling_server: SignalingServer,
    pub transceiver: Transceiver,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SignalingServer {
    pub url: String,
    #[serde(rename = "agentId")]
    pub agent_id: String,
    #[serde(rename = "agentSecret")]
    pub agent_secret: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transceiver {
    #[serde(rename = "model")]
    pub rig_model: u32,
    #[serde(rename = "hamlibDebugLevel", default)]
    pub hamlib_debug_level: Option<HamlibDebugLevel>,
    #[serde(default)]
    pub port: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum HamlibDebugLevel {
    None,
    Bug,
    Err,
    Warn,
    Verbose,
    Trace,
    Cache,
}

impl Display for HamlibDebugLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            HamlibDebugLevel::None => "none",
            HamlibDebugLevel::Bug => "bug",
            HamlibDebugLevel::Err => "err",
            HamlibDebugLevel::Warn => "warn",
            HamlibDebugLevel::Verbose => "verbose",
            HamlibDebugLevel::Trace => "trace",
            HamlibDebugLevel::Cache => "cache",
        };

        f.write_str(value)
    }
}

pub(crate) fn load_config<P: AsRef<Path>>(
    path: P,
) -> Result<Configuration, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Configuration = toml::from_str(&content)?;
    Ok(config)
}
