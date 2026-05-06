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
use log::{debug, error};
use qsp_proto_files::qsp::example::v1::{payload, AgentControlMessage};
use qsp_proto_files::qsp::example::v1::payload::Value;

pub struct CommandSession {
    hello_done: bool,
}

impl CommandSession {
    pub fn new() -> Self {
        Self { hello_done: false }
    }
    pub fn command_received(&mut self, message: &AgentControlMessage) {
        let payload = message
            .payload
            .as_ref()
            .and_then(|payload| payload.value.as_ref());
        let payload_type = agent_control_payload_type(message);

        debug!(
            "AgentControlMessage from DataChannel 'id={}, response_id={}, payload={}'",
            message.id, message.response_id, payload_type,
        );

        match payload {
            Some(payload::Value::Hello(_)) => {
                if !self.hello_done {
                    debug!("Hello received from DataChannel");
                    self.hello_done = true;
                }
            }
            _ if self.hello_done => {
                debug!(
                    "AgentControlMessage '{}' received after hello handshake",
                    payload_type
                );
                CommandSession::command_received_filtered(payload.unwrap());
            }
            _ => {
                error!(
                    "AgentControlMessage '{}' received before hello handshake",
                    payload_type
                );
            }
        }
    }
    fn command_received_filtered(payload: &Value) {
        todo!()
    }
}



fn agent_control_payload_type(message: &AgentControlMessage) -> &'static str {
    match message
        .payload
        .as_ref()
        .and_then(|payload| payload.value.as_ref())
    {
        Some(payload::Value::Hello(_)) => "hello",
        Some(payload::Value::VfoState(_)) => "vfo_state",
        None => "none",
    }
}
