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
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use log::{debug, error, info};
use tokio::sync::Notify;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::io::ogg_reader::OggReader;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc::Error;
use webrtc::peer_connection::RTCPeerConnection;

const OGG_PAGE_DURATION: Duration = Duration::from_millis(20);

pub async fn start_session(client_sdp: String) -> Result<(Arc<RTCPeerConnection>, Box<String>)> {
    let audio_file = "output.ogg";

    if !Path::new(audio_file).exists() {
        return Err(Error::new(format!("audio file: '{}' not exist", audio_file)).into());
    }

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
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    debug!("RTC peer connection created");

    let notify_tx = Arc::new(Notify::new());
    let notify_audio = notify_tx.clone();

    // let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
    // let audio_done_tx = done_tx.clone();


    {
        // Create a audio track
        let audio_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                ..Default::default()
            },
            "audio".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // Add this newly created track to the PeerConnection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        // Read incoming RTCP packets
        // Before these packets are returned they are processed by interceptors. For things
        // like NACK this needs to be called.
        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<()>::Ok(())
        });

        let audio_file_name = audio_file.to_owned();
        tokio::spawn(async move {
            // Open a IVF file and start reading using our IVFReader
            let file = File::open(audio_file_name)?;
            let reader = BufReader::new(file);
            // Open on oggfile in non-checksum mode.
            let (mut ogg, _) = match OggReader::new(reader, true) {
                Ok(tup) => tup,
                Err(err) => {
                    println!("error while opening audio file output.ogg: {}", err);
                    return Err(err.into());
                }
            };
            // Wait for connection established
            let _ = notify_audio.notified().await;

            println!("play audio from disk file output.ogg");

            // It is important to use a time.Ticker instead of time.Sleep because
            // * avoids accumulating skew, just calling time.Sleep didn't compensate for the time spent parsing the data
            // * works around latency issues with Sleep
            let mut ticker = tokio::time::interval(OGG_PAGE_DURATION);

            // Keep track of last granule, the difference is the amount of samples in the buffer
            let mut last_granule: u64 = 0;
            while let Ok((page_data, page_header)) = ogg.parse_next_page() {
                // The amount of samples is the difference between the last and current timestamp
                let sample_count = page_header.granule_position - last_granule;
                last_granule = page_header.granule_position;
                let sample_duration = Duration::from_millis(sample_count * 1000 / 48000);

                audio_track
                    .write_sample(&Sample {
                        data: page_data.freeze(),
                        duration: sample_duration,
                        ..Default::default()
                    })
                    .await?;

                let _ = ticker.tick().await;
            }

            // let _ = audio_done_tx.try_send(());

            Result::<()>::Ok(())
        });
    }
    debug!("Audio track created");

    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_ice_connection_state_change(Box::new(move |connection_state: RTCIceConnectionState| {
            info!("Connection State has changed {}", connection_state);
            if connection_state == RTCIceConnectionState::Connected {
                notify_tx.notify_waiters();
            }
            Box::pin(async {})
        }))
    ;

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            info!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                info!("Peer Connection has gone to failed exiting");
                // let _ = done_tx.try_send(());
            }

            Box::pin(async {})
        }))
    ;

    // Wait for the offer to be pasted
    let offer = RTCSessionDescription::offer(client_sdp).unwrap();
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
    let _ = gather_complete.recv().await;
    debug!("ICE gathering complete");

    let mut agent_sdp: Option<String> = Option::None;
    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        agent_sdp = Some(local_desc.sdp);
        //let json_str = serde_json::to_string(&local_desc)?;
        // let b64 = base64::encode(&json_str);
        // println!("{}", b64);
    } else {
        error!("generate local_description failed!");
        //return Err(Error::new("agent sdp generation failed".to_string()));
    }

    // println!("Press ctrl-c to stop");
    // tokio::select! {
    //     _ = done_rx.recv() => {
    //         println!("received done signal!");
    //     }
    //     _ = tokio::signal::ctrl_c() => {
    //         println!("");
    //     }
    // }
    // ;
    //
    // peer_connection.close().await?;

    Ok((peer_connection, Box::new(agent_sdp.unwrap())))
}