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
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub name: String,
    pub description: String,
    pub signaling_server: SignalingServer,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SignalingServer {
    pub url: String,
    #[serde(rename = "agentId")]
    pub agent_id: String,
    #[serde(rename = "agentSecret")]
    pub agent_secret: String
}


pub(crate) fn load_config<P: AsRef<Path>>(path: P) -> Result<Configuration, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?; // Charger le contenu du fichier
    let config: Configuration = toml::from_str(&content)?; // Désérialiser le contenu TOML
    Ok(config)
}