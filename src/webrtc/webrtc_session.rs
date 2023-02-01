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

use anyhow::Result;
use std::sync::{Arc, Mutex};
use log::{debug, info};
use webrtc::peer_connection::RTCPeerConnection;
use crate::webrtc::webrtc_util::start_session;
use uuid::Uuid;

pub struct WebrtcSession {
    pub agent_rtc_uuid: Arc<String>,
    pub peer_rtc_connection: Option<Arc<RTCPeerConnection>>,
}

pub struct WebrtcSessionManager {
    sessions: Mutex<Vec<WebrtcSession>>
}

impl WebrtcSessionManager {
    pub fn new() -> Self {
        Self { sessions: Mutex::new(Vec::new()) }
    }

    pub async fn add_session(&self, client_sdp: String) -> Result<(Box<String>, Arc<String>)> {
        let (peer_connection, agent_sdp) = start_session(client_sdp).await.expect("Start RTC session failed");
        let session = WebrtcSession {
            agent_rtc_uuid: Arc::new(Uuid::new_v4().to_string()),
            peer_rtc_connection: Some(peer_connection)
        };
        let mut sessions = self.sessions.lock().unwrap();
        let uuid = session.agent_rtc_uuid.clone();

        sessions.push(session);
        Ok((agent_sdp, uuid))
    }

    pub async fn delete_session(&self, uuid: String) {
        let mut sessions = self.sessions.lock().unwrap();
        let position = sessions.iter().position(|s| uuid.eq(s.agent_rtc_uuid.as_str()));

        match position {
            Some(position) => {
                sessions.remove(position);
                debug!("Delete session {}", uuid )
            },
            None => {
                info!("Failed to delete session: uuid {} not found", uuid)
            }
        };


    }
}

