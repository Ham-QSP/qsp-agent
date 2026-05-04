/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::sync::{Arc, Mutex};

use anyhow::Result;
use flume::Receiver;
use log::{debug, info};

use crate::audio::AudioEncodedFrame;
use crate::hardware::audio_io::AudioSessionManager;
use crate::webrtc::webrtc_session::WebrtcSession;

pub struct WebrtcSessionManager {
    sessions: Mutex<Vec<WebrtcSession>>,
    encoded_receiver: Receiver<AudioEncodedFrame>,
    session_manager: Arc<Mutex<AudioSessionManager>>,
}

impl WebrtcSessionManager {
    pub fn new(session_manager: Arc<Mutex<AudioSessionManager>>) -> Self {
        Self {
            sessions: Mutex::new(Vec::new()),
            session_manager: session_manager.clone(),
            encoded_receiver: session_manager.lock().unwrap().get_audio_receiver(),
        }
    }

    pub async fn add_session(&self, client_sdp: String) -> Result<(Box<String>, Arc<String>)> {
        let session = WebrtcSession::create_session(client_sdp, self.encoded_receiver.clone())
            .await
            .expect("Start RTC session failed");
        let mut sessions = self.sessions.lock().unwrap();
        let uuid = session.agent_rtc_uuid.clone();
        let agent_sdp = Box::new(session.agent_sdp.as_ref().clone());

        sessions.push(session);
        Ok((agent_sdp, uuid))
    }

    pub async fn delete_session(&self, uuid: String) {
        let mut sessions = self.sessions.lock().unwrap();
        let position = sessions
            .iter()
            .position(|s| uuid.eq(s.agent_rtc_uuid.as_str()));

        match position {
            Some(position) => {
                sessions.remove(position);
                debug!("Delete session {}", uuid)
            }
            None => {
                info!("Failed to delete session: uuid {} not found", uuid)
            }
        };
    }
}
