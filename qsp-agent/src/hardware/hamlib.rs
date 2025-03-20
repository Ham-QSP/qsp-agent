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
use hamlib::rig::{Hamlib, Rig};
use log::debug;
use crate::configuration::Configuration;
use crate::hardware::error::IOError;

pub struct TransceiverManager {
    hamlib: Hamlib,
    rig: Rig,
}

impl TransceiverManager {
    pub fn new(configuration: Configuration) -> Result<TransceiverManager, IOError> {
        debug!("Hamlib init");
        let mut hamlib = Hamlib::new();

        let rig = hamlib.rig_connect(configuration.transceiver.rig_model).map_err(|e| IOError { message: e.message.to_string() })?;
        Ok(TransceiverManager { hamlib, rig })
        
    }
}
