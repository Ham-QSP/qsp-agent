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

use std::rc::Rc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum AgentSocketMessage {
    #[serde(rename = "SERVER_HELLO")]
    ServerHello { data: ServerDescription },
    #[serde(rename = "AGENT_HELLO")]
    AgentHello { data: Rc<AgentDescription> },
    #[serde(rename = "MESSAGE_ERROR")]
    ErrorMessage {
        #[serde(rename = "errorCode")]
        error_code: u32,
        #[serde(rename = "errorMessage")]
        error_message: String,
        #[serde(rename = "exchangeId")]
        exchange_id: u32,
    },
    #[serde(rename = "CLIENT_INIT")]
    ClientInitMessage { data: ClientInitPayload },
    #[serde(rename = "INIT_RESPONSE")]
    ClientInitResponseMessage { data: ClientInitResponsePayload },
}

#[derive(Serialize, Deserialize)]
pub struct ServerDescription {
    #[serde(rename = "serverType")]
    pub server_type: String,
    pub version: String,
    #[serde(rename = "protocolMajorVersion")]
    pub protocol_major_version: i32,
    #[serde(rename = "protocolMinorVersion")]
    pub protocol_minor_version: i32,
    #[serde(rename = "serverName")]
    pub server_name: String
}

#[derive(Serialize, Deserialize)]
pub struct AgentDescription {
    #[serde(rename = "agentType")]
    pub agent_type: Rc<String>,
    pub version: Rc<String>,
    #[serde(rename = "protocolMajorVersion")]
    pub protocol_major_version: i32,
    #[serde(rename = "protocolMinorVersion")]
    pub protocol_minor_version: i32,
    #[serde(rename = "agentName")]
    pub agent_name: Rc<String>
}

#[derive(Serialize, Deserialize)]
pub struct ClientInitPayload {
    pub sdp: String
}

#[derive(Serialize, Deserialize)]
pub struct ClientInitResponsePayload {
    pub sdp: String
}

pub fn decode_agent_message(message_str: String) -> AgentSocketMessage {
    return serde_json::from_str(&*message_str).expect("Can't decode agent message");
}

