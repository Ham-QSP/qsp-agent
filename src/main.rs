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

use crate::hardware::audio_io::AudioSessionManager;
use std::env;
use std::sync::{Arc, Mutex};

use crate::signaling::signaling_server_manager::SignalingServerManager;
use crate::webrtc::webrtc_session::WebrtcSessionManager;

const APPLICATION_VERSION: &'static str = "0.1.0";
const AGENT_TYPE_NAME: &'static str = "QSP Agent";

#[tokio::main]
async fn main() {
    env_logger::init();

    let audio_session_manager = Arc::new(Mutex::new(AudioSessionManager::new()));
    let webrtc_session_manager = Arc::new(WebrtcSessionManager::new(audio_session_manager));

    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires as argument the signaling server url"));

    let signal_server_session = SignalingServerManager::new(webrtc_session_manager.clone());
    signal_server_session.start(connect_addr).await;

}
