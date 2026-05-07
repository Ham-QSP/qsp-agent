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
use qsp_proto_files::qsp::message::v1::agent_control_message::Message as AgentControlPayload;
use qsp_proto_files::qsp::message::v1::agent_control_message::Message::Transceiver;
use qsp_proto_files::qsp::message::v1::agent_message::AgentMessage as AgentPayload;
use qsp_proto_files::qsp::message::v1::transceiver_message::TransceiverMessage as TransceiverPayload;
use qsp_proto_files::qsp::message::v1::transceiver_message::TransceiverMessage::FrequencyMessage;
use qsp_proto_files::qsp::message::v1::{AgentControlMessage, TrxFrequency};
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
        let payload_type = CommandSession::agent_control_payload_type(message);

        debug!(
            "AgentControlMessage from DataChannel 'payload={}'",
            payload_type,
        );
        match message.message.as_ref() {
            Some(AgentControlPayload::Agent(agent_message)) => {
                match agent_message.agent_message.as_ref() {
                    Some(AgentPayload::Hello(_)) => {
                        if !self.hello_done {
                            debug!("Hello received from DataChannel");
                            self.hello_done = true;
                            tokio::spawn(CommandSession::transceiver_event_loop(
                                self.data_channel.clone(),
                                self.transceiver_manager.clone(),
                            ));
                        }
                    }
                    None => {
                        error!("AgentControlMessage agent payload is empty");
                    }
                }
            }
            Some(AgentControlPayload::Transceiver(transceiver_message)) if self.hello_done => {
                if let Some(payload) = transceiver_message.transceiver_message.as_ref() {
                    CommandSession::command_transceiver_received(payload);
                } else {
                    error!("AgentControlMessage transceiver payload is empty");
                }
            }
            Some(_) => {
                error!(
                    "AgentControlMessage '{}' received before hello handshake",
                    payload_type
                );
            }
            None => {
                error!("AgentControlMessage payload is empty");
            }
        }
    }

    fn command_transceiver_received(payload: &TransceiverPayload) {
        match payload {
            TransceiverPayload::FrequencyMessage(frequency) => {
                debug!(
                    "Frequency command received for VFO {}: {}",
                    frequency.vfo_id, frequency.frequency
                );
            }
        }
    }

    fn agent_control_payload_type(message: &AgentControlMessage) -> &'static str {
        match message.message.as_ref() {
            Some(AgentControlPayload::Agent(agent_message)) => {
                match agent_message.agent_message.as_ref() {
                    Some(AgentPayload::Hello(_)) => "hello",
                    None => "agent_empty",
                }
            }
            Some(AgentControlPayload::Transceiver(transceiver_message)) => {
                match transceiver_message.transceiver_message.as_ref() {
                    Some(TransceiverPayload::FrequencyMessage(_)) => "frequency",
                    None => "transceiver_empty",
                }
            }
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
                message: Some(Transceiver(
                    qsp_proto_files::qsp::message::v1::TransceiverMessage {
                        transceiver_message: Some(FrequencyMessage(TrxFrequency {
                            vfo_id: id as u32,
                            frequency: freq,
                        })),
                    },
                )),
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
