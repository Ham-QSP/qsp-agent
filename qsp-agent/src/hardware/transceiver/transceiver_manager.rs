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

use crate::configuration::{Configuration, HamlibDebugLevel as ConfigHamlibDebugLevel};
use crate::hardware::error::IOError;
use crate::hardware::transceiver::transceiver_state::TransceiverState;
use crate::webrtc::command_session::CommandSession;
use hamlib::hamlib::{Hamlib, RigDebugLevel};
use hamlib::rig::Rig;
use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct TransceiverManager {
    hamlib: Hamlib,
    rig: Mutex<Rig>,
    state: Mutex<TransceiverState>,
    state_polling_interval: Duration,
    command_session: Vec<CommandSession>,
}

impl TransceiverManager {
    pub fn new(configuration: Configuration) -> Result<Arc<TransceiverManager>, IOError> {
        debug!("Hamlib init");
        let mut hamlib = Hamlib::new();
        Hamlib::rig_set_debug_callback(Some(Box::new(|level: RigDebugLevel, message: &str| {
            match level {
                RigDebugLevel::None => trace!("hamlib: {}", message.trim_end()),
                RigDebugLevel::Bug | RigDebugLevel::Err => {
                    error!("hamlib: {}", message.trim_end())
                }
                RigDebugLevel::Warn => warn!("hamlib: {}", message.trim_end()),
                RigDebugLevel::Verbose => info!("hamlib: {}", message.trim_end()),
                RigDebugLevel::Trace | RigDebugLevel::Cache => {
                    debug!("hamlib: {}", message.trim_end())
                }
                RigDebugLevel::Unknown(_) => debug!("hamlib: {}", message.trim_end()),
            }
        })));

        if let Some(level) = configuration.transceiver.hamlib_debug_level {
            let debug_level = level.into();
            debug!("Hamlib debug level: {}", level);
            Hamlib::rig_set_debug(debug_level);
        }

        let rig = hamlib
            .rig_connect(
                configuration.transceiver.rig_model,
                configuration.transceiver.port.clone(),
            )
            .map_err(|e| IOError {
                message: e.message.to_string(),
            })?;

        let manager = Arc::new(TransceiverManager {
            hamlib,
            rig: Mutex::new(rig),
            state: Mutex::new(TransceiverState { mainVfoFreq: 0 }),
            state_polling_interval: Duration::from_millis(
                configuration.transceiver.state_polling_interval_ms,
            ),
            command_session: vec![],
        });
        manager.full_state_update()?;

        let polling_manager = Arc::clone(&manager);
        thread::spawn(move || polling_manager.state_polling_thread_loop());

        Ok(manager)
    }

    pub fn full_state_update(&self) -> Result<(), IOError> {
        let freq = self.rig.lock().unwrap().get_freq(0).map_err(|e| IOError {
            message: e.message.to_string(),
        })?;

        self.state.lock().unwrap().mainVfoFreq = freq as u64;

        Ok(())
    }

    fn state_polling_thread_loop(&self) {
        let mut next_poll = Instant::now();

        loop {
            next_poll += self.state_polling_interval;

            if let Err(error) = self.full_state_update() {
                error!("Failed to update transceiver state: {}", error.message);
            }

            let now = Instant::now();
            if next_poll > now {
                thread::sleep(next_poll - now);
            } else {
                next_poll = now;
            }
        }
    }

    pub fn get_state(&self) -> TransceiverState {
        self.state.lock().unwrap().clone()
    }
}

impl From<ConfigHamlibDebugLevel> for RigDebugLevel {
    fn from(level: ConfigHamlibDebugLevel) -> Self {
        match level {
            ConfigHamlibDebugLevel::None => RigDebugLevel::None,
            ConfigHamlibDebugLevel::Bug => RigDebugLevel::Bug,
            ConfigHamlibDebugLevel::Err => RigDebugLevel::Err,
            ConfigHamlibDebugLevel::Warn => RigDebugLevel::Warn,
            ConfigHamlibDebugLevel::Verbose => RigDebugLevel::Verbose,
            ConfigHamlibDebugLevel::Trace => RigDebugLevel::Trace,
            ConfigHamlibDebugLevel::Cache => RigDebugLevel::Cache,
        }
    }
}
