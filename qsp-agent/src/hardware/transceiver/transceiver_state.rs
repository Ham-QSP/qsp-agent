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

use std::fmt;

#[derive(Clone)]
pub struct TransceiverState {
    pub main_vfo_freq: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransceiverStateMessage {
    pub subsystem: TransceiverSubsystem,
    pub parameter: TransceiverParameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransceiverSubsystem {
    General,
    Vfo { id: u8 },
}

impl fmt::Display for TransceiverSubsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Vfo { id } => write!(f, "vfo:{id}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransceiverParameter {
    Frequency { freq: u64 },
}
