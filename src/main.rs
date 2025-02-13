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

mod audio;
mod signaling;
mod webrtc;
mod hardware;
mod configuration;
mod command_line;

use crate::hardware::audio_io::AudioSessionManager;
use std::sync::{Arc, Mutex};
use log::error;
use crate::configuration::Configuration;
use crate::signaling::signaling_server_manager::SignalingServerManager;
use crate::webrtc::webrtc_session::WebrtcSessionManager;
use clap::Parser;


const APPLICATION_VERSION: &'static str = "0.1.0";
const AGENT_TYPE_NAME: &'static str = "QSP Agent";

#[tokio::main]
async fn main() {
    env_logger::init();
    console_subscriber::init();

    let cli = command_line::Cli::parse();
    
    let config_path = cli.config.unwrap_or("config.toml".parse().unwrap());
    
    match configuration::load_config(config_path) {
        Ok(config) => {
            start_server(config).await;
        }
        Err(e) => error!("Failed to load configuration: {}", e),
    }

}

async fn start_server(config: Configuration) {
    let audio_session_manager = Arc::new(Mutex::new(AudioSessionManager::new()));
    let webrtc_session_manager = Arc::new(WebrtcSessionManager::new(audio_session_manager));

    let signal_server_session = SignalingServerManager::new(config.clone(), webrtc_session_manager.clone());

    signal_server_session.start(config.signaling_server.url).await;
}
