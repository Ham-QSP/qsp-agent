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

use hamlib::hamlib::RigMode;
use std::fmt;

pub type TransceiverMode = RigMode;

#[derive(Clone)]
pub struct TransceiverState {
    pub main_vfo_freq: u64,
    pub main_vfo_mode: Option<TransceiverMode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransceiverStateMessage {
    pub subsystem: TransceiverSubsystem,
    pub parameter: TransceiverParameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransceiverSubsystem {
    Vfo { id: u8 },
}

impl fmt::Display for TransceiverSubsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vfo { id } => write!(f, "vfo:{id}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransceiverParameter {
    Frequency { freq: u64 },
    Mode { mode: TransceiverMode },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransceiverBand {
    Band2200m,
    Band600m,
    Band160m,
    Band80m,
    Band60m,
    Band40m,
    Band30m,
    Band20m,
    Band17m,
    Band15m,
    Band12m,
    Band10m,
    Band6m,
    Band4m,
    Band2m,
    Band125m,
    Band70cm,
    Band33cm,
    Band23cm,
    Band13cm,
    Band9cm,
    Band5cm,
    Band3cm,
    Band12mm,
}

impl TransceiverBand {
    pub fn as_hamlib_name(self) -> Option<&'static str> {
        match self {
            Self::Band2200m => Some("2200m"),
            Self::Band600m => Some("600m"),
            Self::Band160m => Some("160m"),
            Self::Band80m => Some("80m"),
            Self::Band60m => Some("60m"),
            Self::Band40m => Some("40m"),
            Self::Band30m => Some("30m"),
            Self::Band20m => Some("20m"),
            Self::Band17m => Some("17m"),
            Self::Band15m => Some("15m"),
            Self::Band12m => Some("12m"),
            Self::Band10m => Some("10m"),
            Self::Band6m => Some("6m"),
            Self::Band4m => Some("4m"),
            Self::Band2m => Some("2m"),
            Self::Band125m => Some("1.25m"),
            Self::Band70cm => Some("70cm"),
            Self::Band33cm => Some("33cm"),
            Self::Band23cm => Some("23cm"),
            Self::Band13cm => Some("13cm"),
            Self::Band9cm => Some("9cm"),
            Self::Band5cm => Some("5cm"),
            Self::Band3cm => Some("3cm"),
            Self::Band12mm => None,
        }
    }
}
