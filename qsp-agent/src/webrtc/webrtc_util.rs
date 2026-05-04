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
use flume::Receiver;
use log::{debug, error, info};
use prost::Message;
use qsp_proto_files::qsp::example::v1::{payload, AgentControlMessage};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
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
use webrtc::peer_connection::{math_rand_alpha, RTCPeerConnection};
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use crate::audio::AudioEncodedFrame;
use crate::webrtc::command_interpreter::{CommandSession};

pub async fn start_session(
    client_sdp: String,
    encoded_receiver: Receiver<AudioEncodedFrame>,
) -> Result<(Arc<RTCPeerConnection>, Box<String>)> {
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
    tokio::task::Builder::new()
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
    tokio::task::Builder::new()
        .name("Audio sender")
        .spawn(async move {
            // Wait for connection established
            let _ = notify_audio.notified().await;

            debug!("Start thread : Send the audio from the encoder");
            while connected_sender.load(Ordering::Relaxed) {
                match encoded_receiver.recv_async().await {
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
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
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
    }));

    // Wait for the offer to be pasted
    let offer = RTCSessionDescription::offer(client_sdp)?;

    peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        debug!("New DataChannel {d_label} {d_id}");
        let command_session = CommandSession::new();
        //TODO Register session somewhere
        // Register channel opening handling
        Box::pin(async move {
            let d2 = Arc::clone(&d);
            let d_label2 = d_label.clone();
            let d_id2 = d_id;
            d.on_close(Box::new(move || {
                debug!("Data channel closed");
                Box::pin(async {})
            }));

            d.on_open(Box::new(move || {
                debug!("Data channel '{d_label2}'-'{d_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");

                Box::pin(async move {
                    let mut result = Result::<usize>::Ok(0);
                    while result.is_ok() {
                        let timeout = tokio::time::sleep(Duration::from_secs(5));
                        tokio::pin!(timeout);

                        tokio::select! {
                                _ = timeout.as_mut() =>{
                                    let message = math_rand_alpha(15);
                                    debug!("Sending '{message}'");
                                    result = d2.send_text(message).await.map_err(Into::into);
                                }
                            };
                    }
                })
            }));

            // Register protobuf message handling
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                match AgentControlMessage::decode(msg.data) {
                    Ok(message) => {
                        command_session.command_received(&message);
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

    Ok((peer_connection, Box::new(agent_sdp.unwrap())))
}
