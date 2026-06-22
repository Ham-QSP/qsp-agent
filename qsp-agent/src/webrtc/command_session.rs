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
use crate::hardware::transceiver::transceiver_state::{
    TransceiverMode, TransceiverParameter, TransceiverSubsystem,
};
use crate::webrtc::transceiver_mapping::{
    band_to_transceiver_band, transceiver_mode_to_trx_vfo_mode, trx_vfo_mode_to_transceiver_mode,
};
use bytes::Bytes;
use hamlib::hamlib::{RigCaps, RigFrequencyRange};
use hamlib::rig::RigVfoOperation;
use prost::Message;
use qsp_proto_files::qsp::message::v1::agent_control_message::Message as AgentControlPayload;
use qsp_proto_files::qsp::message::v1::agent_control_message::Message::Transceiver;
use qsp_proto_files::qsp::message::v1::agent_message::AgentMessage as AgentPayload;
use qsp_proto_files::qsp::message::v1::transceiver_message::TransceiverMessage as TransceiverPayload;
use qsp_proto_files::qsp::message::v1::transceiver_message::TransceiverMessage::FrequencyMessage;
use qsp_proto_files::qsp::message::v1::transceiver_message::TransceiverMessage::ModeMessage;
use qsp_proto_files::qsp::message::v1::{
    AgentControlMessage, Band, RigFrequencyRangeMessage as ProtoRigFrequencyRange, TrxAntenna,
    TrxCapabilitiesMessage, TrxFrequencyMessage, TrxModeMessage, TrxVfoFlag, TrxVfoMode,
    TrxVfoOperation, TrxVfoOperationMessage,
};
use std::sync::Arc;
use tracing::{debug, error, warn};
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
                            tokio::spawn(CommandSession::send_transceiver_caps(
                                self.data_channel.clone(),
                                self.transceiver_manager.get_caps(),
                            ));
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
            Some(Transceiver(transceiver_message)) if self.hello_done => {
                if let Some(payload) = transceiver_message.transceiver_message.as_ref() {
                    self.command_transceiver_received(payload);
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

    fn command_transceiver_received(&self, payload: &TransceiverPayload) {
        match payload {
            TransceiverPayload::FrequencyMessage(frequency) => {
                debug!(
                    "Frequency command received for VFO {}: {}",
                    frequency.vfo_id, frequency.frequency
                );
                self.transceiver_manager
                    .set_frequency(frequency.vfo_id, frequency.frequency);
            }
            TransceiverPayload::ModeMessage(mode) => {
                let Some(transceiver_mode) = TrxVfoMode::try_from(mode.mode)
                    .ok()
                    .and_then(trx_vfo_mode_to_transceiver_mode)
                else {
                    error!("Unsupported mode command received: {}", mode.mode);
                    return;
                };

                debug!(
                    "Mode command received for VFO {}: {:?}",
                    mode.vfo_id, transceiver_mode
                );
                if let Err(error) = self
                    .transceiver_manager
                    .set_mode(mode.vfo_id, transceiver_mode)
                {
                    error!(
                        "Failed to set VFO {} mode to {:?}: {}",
                        mode.vfo_id, transceiver_mode, error.message
                    );
                }
            }
            TransceiverPayload::BandMessage(band) => {
                let Some(transceiver_band) = Band::try_from(band.band)
                    .ok()
                    .and_then(band_to_transceiver_band)
                else {
                    error!("Unsupported band command received: {}", band.band);
                    return;
                };

                debug!(
                    "Band command received for VFO {}: {:?}",
                    band.vfo_id, transceiver_band
                );
                if let Err(error) = self.transceiver_manager.set_band(transceiver_band) {
                    error!(
                        "Failed to set transceiver band to {:?}: {}",
                        transceiver_band, error.message
                    );
                }
            }
            TransceiverPayload::TrxCapabilitiesMessage(_) => {
                warn!("Transceiver capabilities message received from DataChannel");
            }
            TransceiverPayload::TrxVfoMessage(vfo_operation) => {
                let Some(operation) = rig_vfo_operation_from_trx_vfo_operation(vfo_operation)
                else {
                    error!("Unsupported or missing VFO operation command received");
                    return;
                };

                debug!(
                    "VFO operation command received for VFO {}: {:?}",
                    vfo_operation.vfo_id, operation
                );
                if let Err(error) = self
                    .transceiver_manager
                    .vfo_operation(vfo_operation.vfo_id, operation)
                {
                    error!(
                        "Failed to run VFO {} operation {:?}: {}",
                        vfo_operation.vfo_id, operation, error.message
                    );
                }
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
                    None => "transceiver_empty",
                    Some(FrequencyMessage(_)) => "frequency",
                    Some(ModeMessage(_)) => "mode",
                    Some(TransceiverPayload::BandMessage(_)) => "band",
                    Some(TransceiverPayload::TrxCapabilitiesMessage(_)) => "capabilities",
                    Some(TransceiverPayload::TrxVfoMessage(_)) => "trx_vfo",
                }
            }
            None => "none",
        }
    }

    async fn send_transceiver_caps(data_channel: Arc<RTCDataChannel>, caps: RigCaps) {
        let message = AgentControlMessage {
            message: Some(Transceiver(
                qsp_proto_files::qsp::message::v1::TransceiverMessage {
                    transceiver_message: Some(TransceiverPayload::TrxCapabilitiesMessage(
                        trx_capabilities_from_rig_caps(caps),
                    )),
                },
            )),
        };

        let bytes = Bytes::from(message.encode_to_vec());
        match data_channel.send(&bytes).await {
            Ok(_) => debug!("Sent transceiver capabilities to DataChannel"),
            Err(error) => error!("Failed to send transceiver capabilities to DataChannel: {error}"),
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
                TransceiverParameter::Mode { mode } => {
                    evt_mode_updated(mode, message.subsystem, Arc::clone(&data_channel)).await
                }
            }
        }
    }
}

fn trx_capabilities_from_rig_caps(caps: RigCaps) -> TrxCapabilitiesMessage {
    TrxCapabilitiesMessage {
        rig_model: caps.rig_model,
        model_name: caps.model_name,
        manufacturer_name: caps.manufacturer_name,
        rx_frequency_ranges: caps
            .rx_frequency_ranges
            .into_iter()
            .map(proto_rig_frequency_range_from_rig_frequency_range)
            .collect(),
        tx_frequency_ranges: caps
            .tx_frequency_ranges
            .into_iter()
            .map(proto_rig_frequency_range_from_rig_frequency_range)
            .collect(),
    }
}

fn proto_rig_frequency_range_from_rig_frequency_range(
    range: RigFrequencyRange,
) -> ProtoRigFrequencyRange {
    ProtoRigFrequencyRange {
        list_id: range.region as u32,
        lower_frequency_hz: range.lower_frequency_hz,
        upper_frequency_hz: range.upper_frequency_hz,
        modes: range
            .modes
            .into_iter()
            .map(|mode| transceiver_mode_to_trx_vfo_mode(mode) as i32)
            .collect(),
        vfo: rig_vfo_flags_from_hamlib_vfo(range.vfo),
        antenna: rig_antennas_from_hamlib_antenna(range.antenna),
        label: range.label,
    }
}

fn rig_vfo_operation_from_trx_vfo_operation(
    message: &TrxVfoOperationMessage,
) -> Option<RigVfoOperation> {
    match TrxVfoOperation::try_from(message.operation).ok()? {
        TrxVfoOperation::Unspecified => None,
        TrxVfoOperation::BandUp => Some(RigVfoOperation::BandUp),
        TrxVfoOperation::BandDown => Some(RigVfoOperation::BandDown),
    }
}

fn rig_antennas_from_hamlib_antenna(antenna: u32) -> Vec<i32> {
    const HAMLIB_ANTENNA_FLAGS: &[(u32, TrxAntenna)] = &[
        (1u32 << 0, TrxAntenna::TrxAntenna1),
        (1u32 << 1, TrxAntenna::TrxAntenna2),
        (1u32 << 2, TrxAntenna::TrxAntenna3),
        (1u32 << 3, TrxAntenna::TrxAntenna4),
        (1u32 << 4, TrxAntenna::TrxAntenna5),
        (1u32 << 5, TrxAntenna::TrxAntenna6),
        (1u32 << 6, TrxAntenna::TrxAntenna7),
        (1u32 << 7, TrxAntenna::TrxAntenna8),
    ];

    if antenna == 0 {
        return vec![TrxAntenna::Unspecified as i32];
    }

    let known_mask = HAMLIB_ANTENNA_FLAGS
        .iter()
        .fold(0u32, |mask, (bit, _)| mask | *bit);
    let unknown_mask = antenna & !known_mask;
    if unknown_mask != 0 {
        warn!("Unsupported Hamlib antenna flags in capabilities: 0x{unknown_mask:08x}");
    }

    HAMLIB_ANTENNA_FLAGS
        .iter()
        .filter_map(|(bit, antenna_flag)| {
            if antenna & *bit != 0 {
                Some(*antenna_flag as i32)
            } else {
                None
            }
        })
        .collect()
}

fn rig_vfo_flags_from_hamlib_vfo(vfo: u32) -> Vec<i32> {
    const HAMLIB_VFO_FLAGS: &[(u32, TrxVfoFlag)] = &[
        (1u32 << 0, TrxVfoFlag::A),
        (1u32 << 1, TrxVfoFlag::B),
        (1u32 << 2, TrxVfoFlag::C),
        (1u32 << 3, TrxVfoFlag::SubC),
        (1u32 << 4, TrxVfoFlag::MainC),
        (1u32 << 5, TrxVfoFlag::Other),
        (1u32 << 21, TrxVfoFlag::SubA),
        (1u32 << 22, TrxVfoFlag::SubB),
        (1u32 << 23, TrxVfoFlag::MainA),
        (1u32 << 24, TrxVfoFlag::MainB),
        (1u32 << 25, TrxVfoFlag::Sub),
        (1u32 << 26, TrxVfoFlag::Main),
        (1u32 << 27, TrxVfoFlag::Vfo),
        (1u32 << 28, TrxVfoFlag::Mem),
        (1u32 << 29, TrxVfoFlag::Curr),
        (1u32 << 30, TrxVfoFlag::TxFlag),
    ];

    if vfo == 0 {
        return vec![TrxVfoFlag::Unspecified as i32];
    }

    let known_mask = HAMLIB_VFO_FLAGS
        .iter()
        .fold(0u32, |mask, (bit, _)| mask | *bit);
    let unknown_mask = vfo & !known_mask;
    if unknown_mask != 0 {
        warn!("Unsupported Hamlib VFO flags in capabilities: 0x{unknown_mask:08x}");
    }

    HAMLIB_VFO_FLAGS
        .iter()
        .filter_map(|(bit, flag)| {
            if vfo & *bit != 0 {
                Some(*flag as i32)
            } else {
                None
            }
        })
        .collect()
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
                        transceiver_message: Some(FrequencyMessage(TrxFrequencyMessage {
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

async fn evt_mode_updated(
    mode: TransceiverMode,
    transceiver_subsystem: TransceiverSubsystem,
    data_channel: Arc<RTCDataChannel>,
) {
    match transceiver_subsystem {
        TransceiverSubsystem::Vfo { id } => {
            let mode_value = transceiver_mode_to_trx_vfo_mode(mode);

            let message = AgentControlMessage {
                message: Some(Transceiver(
                    qsp_proto_files::qsp::message::v1::TransceiverMessage {
                        transceiver_message: Some(ModeMessage(TrxModeMessage {
                            vfo_id: id as u32,
                            mode: mode_value as i32,
                        })),
                    },
                )),
            };

            let bytes = Bytes::from(message.encode_to_vec());
            match data_channel.send(&bytes).await {
                Ok(_) => debug!("Sent VFO {id} mode update to DataChannel: {mode:?}"),
                Err(error) => {
                    error!("Failed to send VFO {id} mode update to DataChannel: {error}")
                }
            }
        }
        TransceiverSubsystem::General => {
            error!(
                "Received unknown transceiver subsystem ({}) to set mode",
                transceiver_subsystem
            );
        }
    }
}
