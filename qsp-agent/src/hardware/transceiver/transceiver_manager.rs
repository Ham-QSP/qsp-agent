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

use crate::configuration::{Configuration, Transceiver};
use crate::hardware::error::IOError;
use crate::hardware::transceiver::transceiver_state::TransceiverState;
use hamlib::rig::{Hamlib, Rig};
use log::debug;
use std::sync::{Arc, Mutex};

pub struct TransceiverManager {
    hamlib: Hamlib,
    rig: Rig,
    state: Mutex<TransceiverState>,
}

impl TransceiverManager {
    pub fn new(configuration: Configuration) -> Result<TransceiverManager, IOError> {
        debug!("Hamlib init");
        let mut hamlib = Hamlib::new();

        let rig = hamlib
            .rig_connect(configuration.transceiver.rig_model)
            .map_err(|e| IOError {
                message: e.message.to_string(),
            })?;
        
        let manager = TransceiverManager {
            hamlib,
            rig,
            state: Mutex::new(TransceiverState { mainVfoFreq: 0 }),
        };
        manager.full_state_update()?;
        Ok(manager)
    }

    pub fn full_state_update(&self) -> Result<(), IOError> {
        let mut state  = self.state.lock().unwrap();
        match self.rig.get_freq(0)  {
            Ok(freq) => {state.mainVfoFreq = freq as u64}
            Err(e) => { return Err(IOError { message: e.message.to_string() })}
        };
        
        Ok(())
    }
    pub fn get_state(&self) -> TransceiverState {
        self.state.lock().unwrap().clone()
    }
}
