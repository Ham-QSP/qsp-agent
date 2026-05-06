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
use crate::hardware::transceiver::transceiver_manager::TransceiverManager;
use crate::hardware::transceiver::transceiver_state::{TransceiverParameter, TransceiverSubsystem};
use bytes::Bytes;
use log::{debug, error};
use prost::Message;
use qsp_proto_files::qsp::example::v1::payload::Value;
use qsp_proto_files::qsp::example::v1::{payload, AgentControlMessage, Payload, VfoState};
use std::sync::Arc;
use webrtc::data_channel::RTCDataChannel;

pub struct CommandSession {
    hello_done: bool,
    data_channel: Arc<RTCDataChannel>,
    transceiver_manager: Arc<TransceiverManager>,
}

impl CommandSession {
    pub fn new(
        data_channel: Arc<RTCDataChannel>,
        transceiver_manager: Arc<TransceiverManager>,
    ) -> Self {
        Self {
            hello_done: false,
            data_channel,
            transceiver_manager,
        }
    }
    pub fn command_received(&mut self, message: &AgentControlMessage) {
        let payload = message
            .payload
            .as_ref()
            .and_then(|payload| payload.value.as_ref());
        let payload_type = CommandSession::agent_control_payload_type(message);

        debug!(
            "AgentControlMessage from DataChannel 'id={}, response_id={}, payload={}'",
            message.id, message.response_id, payload_type,
        );

        match payload {
            Some(Value::Hello(_)) => {
                if !self.hello_done {
                    debug!("Hello received from DataChannel");
                    self.hello_done = true;
                    tokio::spawn(CommandSession::transceiver_event_loop(
                        self.data_channel.clone(),
                        self.transceiver_manager.clone(),
                    ));
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

    fn agent_control_payload_type(message: &AgentControlMessage) -> &'static str {
        match message
            .payload
            .as_ref()
            .and_then(|payload| payload.value.as_ref())
        {
            Some(Value::Hello(_)) => "hello",
            Some(Value::VfoState(_)) => "vfo_state",
            None => "none",
        }
    }

    async fn transceiver_event_loop(
        data_channel: Arc<RTCDataChannel>,
        transceiver_manager: Arc<TransceiverManager>,
    ) {
        debug!("CommandSession transceiver event loop started");
        let mut receiver = transceiver_manager.add_state_update_receiver();
        transceiver_manager.send_current_state();
        while let Some(message) = receiver.recv().await {
            match message.parameter {
                TransceiverParameter::Frequency { freq } => {
                    evt_freq_updated(freq, message.subsystem, Arc::clone(&data_channel)).await
                }
            }
        }
    }
}

async fn evt_freq_updated(
    freq: u64,
    transceiver_subsystem: TransceiverSubsystem,
    data_channel: Arc<RTCDataChannel>,
) {
    match transceiver_subsystem {
        TransceiverSubsystem::Vfo { id } => {
            let message = AgentControlMessage {
                id: 0,
                response_id: 0,
                payload: Some(Payload {
                    value: Some(Value::VfoState(VfoState {
                        frequency: freq,
                        vfo_mode: String::new(),
                    })),
                }),
            };

            let bytes = Bytes::from(message.encode_to_vec());
            match data_channel.send(&bytes).await {
                Ok(_) => debug!("Sent VFO {id} frequency update to DataChannel: {freq}"),
                Err(error) => {
                    error!("Failed to send VFO {id} frequency update to DataChannel: {error}")
                }
            }
        }
        TransceiverSubsystem::General => {
            error!(
                "Received unknown transceiver subsystem ({}) to set frequency",
                transceiver_subsystem
            );
        }
    }
}
