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

use std::sync::{Arc};
use std::thread;
use tokio::time::Duration;

use bytes::Bytes;
use cpal::{ SampleFormat, SampleRate, SupportedStreamConfig, SupportedStreamConfigRange};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use flume::Receiver;
use log::{debug, info};
use crate::audio::{AudioEncodedFrame, AudioFrame};

pub struct AudioManager {}

impl AudioManager {
    pub fn new() -> Self {
        Self {}
    }
    pub fn start(self) -> (Stream, Receiver<AudioEncodedFrame>) {
        let (sender, frame_receiver) = flume::bounded::<AudioFrame>(3);
        let (encoded_sender, encoded_receiver) = flume::bounded::<AudioEncodedFrame>(3);

        thread::spawn(move || {
            // We just handle 48khz, to handle other sample rates like 44.1khz you need to use a resampler.
            let mut encoder =
                opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip).unwrap();

            loop {
                let AudioFrame { data } = frame_receiver.recv().unwrap();

                let sample_count = data.len() as u64;
                // sample duration
                let duration = Duration::from_millis(sample_count * 1000 / 48000);
                let encoded = encoder
                    .encode_vec_float(&data, 1024)
                    .expect("Failed to encode");
                let bytes = Bytes::from(encoded);

                encoded_sender
                    .send(AudioEncodedFrame { bytes, duration })
                    .unwrap();
            }
        });

        let host = cpal::default_host();

        // Set up the input device and stream with the default input config.
        let device = host.default_input_device()
            .expect("failed to find input device");

        info!("Audio input device: {}", device.name().unwrap());

        let input_configs = match device.supported_input_configs() {
            Ok(f) => f.collect(),
            Err(e) => {
                println!("    Error getting supported input configs: {:?}", e);
                Vec::new()
            }
        };
        let config = AudioManager::find_audio_config(input_configs).unwrap();

        debug!("Audio default input config: {:?}", config);


        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let config = config.config();
        // until it is 960
        let mut buffer: Vec<f32> = Vec::new();

        // assume cpal::SampleFormat::F32
        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    for &sample in data {
                        buffer.push(sample.clone());
                        if buffer.len() == 960 {
                            sender
                                .send(AudioFrame {
                                    data: Arc::new(buffer.to_owned()),
                                })
                                .expect("Failed to send raw frame to the encoder");
                            // Create a new vec
                            buffer.clear();
                        }
                    }
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        return (stream, encoded_receiver);
    }

    fn find_audio_config(configs: Vec<SupportedStreamConfigRange>) -> Option<SupportedStreamConfig> {
        return if !configs.is_empty() {
            let configs = configs.into_iter()
                .filter(|c| {
                    return c.min_sample_rate().0 <= 48000
                        && c.max_sample_rate().0 >= 48000
                        && c.sample_format() == SampleFormat::F32
                        && c.channels() == 1;
                });
            let x: Vec<SupportedStreamConfigRange> = configs.collect();
            return x.first().map(|range|
                SupportedStreamConfig::new(1,
                                           SampleRate(48000),
                                           range.buffer_size().clone(),
                                           SampleFormat::F32)
            );
        } else { None };
    }
}
