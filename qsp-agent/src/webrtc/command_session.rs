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
use qsp_proto_files::qsp::message::v1::{AgentControlMessage, Band, TrxFrequency, TrxVfoMode};
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
                let Some(mode_name) = TrxVfoMode::try_from(mode.mode)
                    .ok()
                    .and_then(trx_vfo_mode_to_hamlib)
                else {
                    error!("Unsupported mode command received: {}", mode.mode);
                    return;
                };

                debug!(
                    "Mode command received for VFO {}: {}",
                    mode.vfo_id, mode_name
                );
                if let Err(error) = self.transceiver_manager.set_mode(mode.vfo_id, mode_name) {
                    error!(
                        "Failed to set VFO {} mode to {}: {}",
                        mode.vfo_id, mode_name, error.message
                    );
                }
            }
            TransceiverPayload::BandMessage(band) => {
                let Some(band_name) = Band::try_from(band.band).ok().and_then(band_to_hamlib_band)
                else {
                    error!("Unsupported band command received: {}", band.band);
                    return;
                };

                debug!(
                    "Band command received for VFO {}: {}",
                    band.vfo_id, band_name
                );
                if let Err(error) = self.transceiver_manager.set_band(band_name) {
                    error!(
                        "Failed to set transceiver band to {}: {}",
                        band_name, error.message
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
                    Some(TransceiverPayload::FrequencyMessage(_)) => "frequency",
                    Some(TransceiverPayload::ModeMessage(_)) => "mode",
                    Some(TransceiverPayload::BandMessage(_)) => "band",
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

fn trx_vfo_mode_to_hamlib(mode: TrxVfoMode) -> Option<&'static str> {
    match mode {
        TrxVfoMode::Unspecified => None,
        TrxVfoMode::Cw => Some("CW"),
        TrxVfoMode::Usb => Some("USB"),
        TrxVfoMode::Lsb => Some("LSB"),
        TrxVfoMode::Rtty => Some("RTTY"),
        TrxVfoMode::Fm => Some("FM"),
        TrxVfoMode::Wfm => Some("WFM"),
        TrxVfoMode::Cwr => Some("CWR"),
        TrxVfoMode::Rttyr => Some("RTTYR"),
        TrxVfoMode::Ams => Some("AMS"),
        TrxVfoMode::Pktlsb => Some("PKTLSB"),
        TrxVfoMode::Pktusb => Some("PKTUSB"),
        TrxVfoMode::Pktfm => Some("PKTFM"),
        TrxVfoMode::Ecssusb => Some("ECSSUSB"),
        TrxVfoMode::Exsslsb => Some("ECSSLSB"),
        TrxVfoMode::Fax => Some("FAX"),
        TrxVfoMode::Sam => Some("SAM"),
        TrxVfoMode::Dsb => Some("DSB"),
        TrxVfoMode::Fmn => Some("FMN"),
        TrxVfoMode::Pktam => Some("PKTAM"),
        TrxVfoMode::P25 => Some("P25"),
        TrxVfoMode::Dstar => Some("DSTAR"),
        TrxVfoMode::Dpmr => Some("DPMR"),
        TrxVfoMode::Nxdnvn => Some("NXDNVN"),
        TrxVfoMode::NxdnN => Some("NXDNN"),
        TrxVfoMode::Dcr => Some("DCR"),
        TrxVfoMode::Amn => Some("AMN"),
        TrxVfoMode::Psk => Some("PSK"),
        TrxVfoMode::Pskr => Some("PSKR"),
        TrxVfoMode::Dd => Some("DD"),
        TrxVfoMode::C4fm => Some("C4FM"),
        TrxVfoMode::Pktfmn => Some("PKTFMN"),
        TrxVfoMode::Spec => Some("SPEC"),
        TrxVfoMode::Cwn => Some("CWN"),
        TrxVfoMode::Am => Some("AM"),
    }
}

fn band_to_hamlib_band(band: Band) -> Option<&'static str> {
    match band {
        Band::Unspecified => None,
        Band::Band2200m => Some("2200m"),
        Band::Band600m => Some("600m"),
        Band::Band160m => Some("160m"),
        Band::Band80m => Some("80m"),
        Band::Band60m => Some("60m"),
        Band::Band40m => Some("40m"),
        Band::Band30m => Some("30m"),
        Band::Band20m => Some("20m"),
        Band::Band17m => Some("17m"),
        Band::Band15m => Some("15m"),
        Band::Band12m => Some("12m"),
        Band::Band1om => Some("10m"),
        Band::Band6m => Some("6m"),
        Band::Band4m => Some("4m"),
        Band::Band2m => Some("2m"),
        Band::Band125m => Some("1.25m"),
        Band::Band70cm => Some("70cm"),
        Band::Band33cm => Some("33cm"),
        Band::Band23cm => Some("23cm"),
        Band::Band13cm => Some("13cm"),
        Band::Band9cm => Some("9cm"),
        Band::Band5cm => Some("5cm"),
        Band::Band3cm => Some("3cm"),
        Band::Band12mm => None,
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
