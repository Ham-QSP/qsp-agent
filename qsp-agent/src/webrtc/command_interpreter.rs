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
use log::debug;
use qsp_proto_files::qsp::example::v1::{payload, AgentControlMessage};

pub struct CommandSession {
    hello_done: bool,
}

impl CommandSession {
    pub fn new() -> Self {
        Self { hello_done: false }
    }
    pub fn command_received(&self, message: &AgentControlMessage) {
        debug!(
            "AgentControlMessage from DataChannel 'id={}, response_id={}",
            message.id, message.response_id,
        );
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
