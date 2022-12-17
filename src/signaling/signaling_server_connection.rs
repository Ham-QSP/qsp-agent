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

use tungstenite::{connect, Message};

use crate::{AGENT_TYPE_NAME, APPLICATION_VERSION};
use crate::signaling::message_decoder::{AgentDescription, AgentSocketMessage, ClientInitResponsePayload, decode_agent_message};

const PROTOCOL_VERSION_MAJOR: i32 = 0;
const PROTOCOL_VERSION_MINOR: i32 = 1;

#[derive(thiserror::Error, Debug)]
pub enum SignalingServerError {
    #[error("Can't connect the signaling server")]
    ConnectionFailed(#[from] tungstenite::Error),
    #[error("Can't connect the signaling server")]
    ProtocolFormatError(#[from] serde_json::Error),
}

pub struct SignalingServerConnection {}


impl SignalingServerConnection {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self, url: url::Url) -> Result<(), SignalingServerError> {
        let (mut socket, _) =
            connect(url)?;

        loop {
            let msg_str = socket.read_message()?;
            let msg = decode_agent_message(msg_str.into_text()?);
            let tx_message = self.process_message(msg);
            if let Some(tx_message) = tx_message {
                let tx_message_str = serde_json::to_string(&tx_message)?;
                socket.write_message(Message::Text(tx_message_str))?;
            }
        }
    }

    fn process_message(&mut self, message: AgentSocketMessage) -> Option<AgentSocketMessage> {
        match message {
            AgentSocketMessage::ServerHello { data } => {
                println!("Received command Server hello. Server is '{}'", data.server_name);
                let agent_description = AgentDescription {
                    agent_type: AGENT_TYPE_NAME.to_string(),
                    version: APPLICATION_VERSION.to_string(),
                    protocol_major_version: PROTOCOL_VERSION_MAJOR,
                    protocol_minor_version: PROTOCOL_VERSION_MINOR,
                    agent_name: "In development".to_string(),
                };
                Some(AgentSocketMessage::AgentHello { data: agent_description })
            }
            AgentSocketMessage::ClientInitMessage { data } => {
                println!("Received client init");
                Some(AgentSocketMessage::ClientInitResponseMessage {
                    data: ClientInitResponsePayload { sdp: "Mocked agent sdp payload".to_string() }
                })
            }
            _ => {
                println!("Received unexpected command type");
                Some(AgentSocketMessage::ErrorMessage {
                    error_code: 102,
                    error_message: "Agent received invalid command name".to_string(),
                    exchange_id: 0,
                })
            }
        }
    }
}