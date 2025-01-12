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
use std::sync::Arc;
use log::{debug, info};

use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{AGENT_TYPE_NAME, APPLICATION_VERSION};
use crate::signaling::message_decoder::{AgentDescription, AgentSocketMessage, ClientInitResponsePayload, decode_agent_message};
use crate::webrtc::webrtc_session::{WebrtcSessionManager};

const PROTOCOL_VERSION_MAJOR: i32 = 0;
const PROTOCOL_VERSION_MINOR: i32 = 1;

#[derive(thiserror::Error, Debug)]
pub enum SignalingServerError {
    #[error("Can't connect the signaling server")]
    ConnectionFailed(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Can't connect the signaling server")]
    ProtocolFormatError(#[from] serde_json::Error),
}

pub struct SignalingServerSession {
    agent_description: Arc<AgentDescription>,
}

#[derive(Clone)]
pub struct SignalingServerManager {
    agent_description: Arc<AgentDescription>,
    webrtc_session_manager: Arc<WebrtcSessionManager>,
}

impl SignalingServerManager {
    pub fn new(webrtc_session_manager: Arc<WebrtcSessionManager>) -> Self {
        Self {
            agent_description: Arc::new(AgentDescription {
                agent_type: Arc::new(AGENT_TYPE_NAME.to_string()),
                version: Arc::new(APPLICATION_VERSION.to_string()),
                protocol_major_version: PROTOCOL_VERSION_MAJOR,
                protocol_minor_version: PROTOCOL_VERSION_MINOR,
                agent_name: Arc::new("Agent in development".to_string()),
            }),
            webrtc_session_manager
        }
    }
    pub async fn start(self, url: String) {
        let connection = tokio::spawn(self.connect(url));
        connection.await.expect("An error occur in session management").expect("TODO: panic message");
    }

    async fn connect(self, url: String) -> Result<(), SignalingServerError> {
        let (ws_stream, _) = connect_async(&url).await?;
        let (write, read) = ws_stream.split();

        let session = Arc::new(SignalingServerManager::create_session(self.agent_description.clone()));
        let (send_tx, send_rx) = futures_channel::mpsc::unbounded();
        let send_to_ws = send_rx.map(Ok).forward(write);

        let receive_from_ws = {
            read.for_each(|message| async {
                let msg_str = message.expect("error message");
                let msg = decode_agent_message(msg_str.into_text().expect("Deserialization error"));
                let tx_message = SignalingServerManager::process_message(
                    self.webrtc_session_manager.clone(),
                    session.clone(),
                    msg).await.unwrap();
                if let Some(tx_message) = tx_message {
                    let tx_message_str = serde_json::to_string(&tx_message).expect("Serialization error");
                    send_tx.unbounded_send(Message::Text(tx_message_str)).expect("Can't send message");
                }
            })
        };

        pin_mut!(send_to_ws, receive_from_ws);
        future::select(send_to_ws, receive_from_ws).await;

        Ok(())
    }

    fn create_session(agent_description: Arc<AgentDescription>) -> SignalingServerSession {
        SignalingServerSession { agent_description }
    }


    async fn process_message(webrtc_session_manager: Arc<WebrtcSessionManager>,
                             session: Arc<SignalingServerSession>,
                             message: AgentSocketMessage) -> Result<Option<AgentSocketMessage>> {
        match message {
            AgentSocketMessage::ServerHello { data } => {
                info!("Got server hello. Server name is '{}'", data.server_name);
                Ok(Some(AgentSocketMessage::AgentHello { data: session.agent_description.clone() }))
            }
            AgentSocketMessage::ClientInitMessage { data } => {
                info!("Received client init");
                let (agent_sdp, uuid) = webrtc_session_manager.add_session( data.sdp).await?;
                debug!("Client init complete. Send client init response with uuid={}", uuid);
                Ok(Some(AgentSocketMessage::ClientInitResponseMessage {
                    data: ClientInitResponsePayload { sdp: agent_sdp.to_string(), agent_session_uuid: uuid }
                }))
            }
            _ => {
                info!("Received unexpected command type");
                Ok(Some(AgentSocketMessage::ErrorMessage {
                    error_code: 102,
                    error_message: "Agent received invalid command name".to_string(),
                    exchange_id: 0,
                }))
            }
        }

    }
}