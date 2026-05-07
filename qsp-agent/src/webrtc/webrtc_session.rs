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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use flume::Receiver;
use log::{debug, error, info};
use prost::Message;
use qsp_proto_files::qsp::message::v1::AgentControlMessage;
use tokio::sync::Notify;
use uuid::Uuid;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use crate::audio::AudioEncodedFrame;
use crate::hardware::transceiver::transceiver_manager::TransceiverManager;
use crate::webrtc::command_session::CommandSession;

pub struct WebrtcSession {
    pub agent_rtc_uuid: Arc<String>,
    pub peer_rtc_connection: Option<Arc<RTCPeerConnection>>,
    encoded_receiver: Receiver<AudioEncodedFrame>,
    command_session: Arc<Mutex<Option<CommandSession>>>,
    pub(crate) agent_sdp: Arc<String>,
}

impl WebrtcSession {
    pub(super) async fn create_session(
        client_sdp: String,
        encoded_receiver: Receiver<AudioEncodedFrame>,
        transceiver_manager: Arc<TransceiverManager>,
    ) -> Result<WebrtcSession> {
        debug!("Starting webRTC session");
        // Create a MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();

        m.register_default_codecs()?;

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut m)?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Prepare the configuration
        let rtc_config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);
        debug!("RTC peer connection created");

        let notify_tx = Arc::new(Notify::new());
        let notify_audio = notify_tx.clone();

        // Create a audio track
        let audio_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                //channels: 1,
                ..Default::default()
            },
            "audio".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // Add this newly created track to the PeerConnection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        let connected = Arc::new(AtomicBool::new(true));
        // Read incoming RTCP packets
        // Before these packets are returned they are processed by interceptors. For things
        // like NACK this needs to be called.
        let connected_rtcp_reader = connected.clone();
        let _ = tokio::task::Builder::new()
            .name("RTCP Reader")
            .spawn(async move {
                let mut rtcp_buf = vec![0u8; 1500];
                debug!("Start thread: RTCP reader");
                while connected_rtcp_reader.load(Ordering::Relaxed) {
                    match rtp_sender.read(&mut rtcp_buf).await {
                        Ok(_size) => {}
                        Err(e) => {
                            error!("RTCP send thread error: {}", e);
                            break;
                        }
                    }
                }
                debug!("End thread: RTCP reader");
            });

        // SENDER
        let connected_sender = connected.clone();
        let audio_receiver = encoded_receiver.clone();
        let _ = tokio::task::Builder::new()
            .name("Audio sender")
            .spawn(async move {
                // Wait for connection established
                let _ = notify_audio.notified().await;

                debug!("Start thread : Send the audio from the encoder");
                while connected_sender.load(Ordering::Relaxed) {
                    match audio_receiver.recv_async().await {
                        Ok(frame) => {
                            audio_track
                                .write_sample(&Sample {
                                    data: frame.bytes,
                                    duration: frame.duration,
                                    ..Default::default()
                                })
                                .await?;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                debug!("End send audio thread");

                Result::<()>::Ok(())
            });

        debug!("Audio track created");

        // Set the handler for ICE connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection.on_ice_connection_state_change(Box::new(
            move |connection_state: RTCIceConnectionState| {
                info!("Ice Connection State has changed {}", connection_state);
                if connection_state == RTCIceConnectionState::Connected {
                    notify_tx.notify_waiters();
                }
                Box::pin(async {})
            },
        ));

        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        let connected_store = connected.clone();
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                debug!("Peer Connection State has changed: {}", s);
                // Remove session
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                info!("Peer Connection has gone to failed exiting");
                if s == RTCPeerConnectionState::Failed {
                    // let _ = done_tx.try_send(());
                    connected_store.store(false, Ordering::Relaxed);
                }

                Box::pin(async {})
            },
        ));

        let command_session = Arc::new(Mutex::new(None));
        Self::register_data_channel_handler(
            &peer_connection,
            Arc::clone(&command_session),
            transceiver_manager,
        );

        // Wait for the offer to be pasted
        let offer = RTCSessionDescription::offer(client_sdp)?;
        debug!("client SDP offer created");
        // Set the remote SessionDescription
        peer_connection.set_remote_description(offer).await?;
        debug!("client SDP set");

        // Create an answer
        let answer = peer_connection.create_answer(None).await?;
        debug!("RTC answer created");

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(answer).await?;
        debug!("Local description set");

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        //TODO Check this comment about OnIceCandidate
        let _ = gather_complete.recv().await;
        debug!("ICE gathering complete");

        let mut agent_sdp: Option<String> = Option::None;
        // Get the answer to return to the server, then to the browser
        if let Some(local_desc) = peer_connection.local_description().await {
            agent_sdp = Some(local_desc.sdp);
        } else {
            error!("generate local_description failed!");
        }

        let session = WebrtcSession {
            agent_rtc_uuid: Arc::new(Uuid::new_v4().to_string()),
            peer_rtc_connection: Some(peer_connection),
            encoded_receiver,
            command_session,
            agent_sdp: Arc::new(agent_sdp.unwrap()),
        };
        Ok(session)
    }

    fn register_data_channel_handler(
        peer_connection: &Arc<RTCPeerConnection>,
        command_session_store: Arc<Mutex<Option<CommandSession>>>,
        transceiver_manager: Arc<TransceiverManager>,
    ) {
        peer_connection.on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
            let d_label = data_channel.label().to_owned();
            let d_id = data_channel.id();
            debug!("New DataChannel {d_label} {d_id}");
            *command_session_store.lock().unwrap() = Some(CommandSession::new(data_channel.clone(), transceiver_manager.clone()));
            let command_session_for_messages = Arc::clone(&command_session_store);

            Box::pin(async move {
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                data_channel.on_close(Box::new(move || {
                    debug!("Data channel closed");
                    Box::pin(async {})
                }));

                data_channel.on_open(Box::new(move || {
                    debug!("Data channel '{d_label2}'-'{d_id2}' open. ");

                    Box::pin(async {})
                }));

                // Register protobuf message handling
                let command_session = Arc::clone(&command_session_for_messages);
                data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
                    match AgentControlMessage::decode(msg.data) {
                        Ok(message) => {
                            if let Some(command_session) = command_session.lock().unwrap().as_mut() {
                                command_session.command_received(&message);
                            }
                        }
                        Err(error) => {
                            error!(
                            "Failed to decode AgentControlMessage from DataChannel '{d_label}': {error}"
                        );
                        }
                    }

                    Box::pin(async {})
                }));
            })
        }));
    }
}
