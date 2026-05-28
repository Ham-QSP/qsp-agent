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
    pub main_vfo_mode: Option<TransceiverMode>,
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
    Mode { mode: TransceiverMode },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransceiverMode {
    Cw,
    Usb,
    Lsb,
    Rtty,
    Fm,
    Wfm,
    Cwr,
    Rttyr,
    Ams,
    Pktlsb,
    Pktusb,
    Pktfm,
    Ecssusb,
    Ecsslsb,
    Fax,
    Sam,
    Dsb,
    Fmn,
    Pktam,
    P25,
    Dstar,
    Dpmr,
    Nxdnvn,
    NxdnN,
    Dcr,
    Amn,
    Psk,
    Pskr,
    Dd,
    C4fm,
    Pktfmn,
    Spec,
    Cwn,
    Am,
}

impl TransceiverMode {
    pub fn as_hamlib_name(self) -> &'static str {
        match self {
            Self::Cw => "CW",
            Self::Usb => "USB",
            Self::Lsb => "LSB",
            Self::Rtty => "RTTY",
            Self::Fm => "FM",
            Self::Wfm => "WFM",
            Self::Cwr => "CWR",
            Self::Rttyr => "RTTYR",
            Self::Ams => "AMS",
            Self::Pktlsb => "PKTLSB",
            Self::Pktusb => "PKTUSB",
            Self::Pktfm => "PKTFM",
            Self::Ecssusb => "ECSSUSB",
            Self::Ecsslsb => "ECSSLSB",
            Self::Fax => "FAX",
            Self::Sam => "SAM",
            Self::Dsb => "DSB",
            Self::Fmn => "FMN",
            Self::Pktam => "PKTAM",
            Self::P25 => "P25",
            Self::Dstar => "DSTAR",
            Self::Dpmr => "DPMR",
            Self::Nxdnvn => "NXDNVN",
            Self::NxdnN => "NXDNN",
            Self::Dcr => "DCR",
            Self::Amn => "AMN",
            Self::Psk => "PSK",
            Self::Pskr => "PSKR",
            Self::Dd => "DD",
            Self::C4fm => "C4FM",
            Self::Pktfmn => "PKTFMN",
            Self::Spec => "SPEC",
            Self::Cwn => "CWN",
            Self::Am => "AM",
        }
    }

    pub fn from_hamlib_name(mode: &str) -> Option<Self> {
        match mode.trim().to_ascii_uppercase().as_str() {
            "CW" => Some(Self::Cw),
            "USB" => Some(Self::Usb),
            "LSB" => Some(Self::Lsb),
            "RTTY" => Some(Self::Rtty),
            "FM" => Some(Self::Fm),
            "WFM" => Some(Self::Wfm),
            "CWR" => Some(Self::Cwr),
            "RTTYR" => Some(Self::Rttyr),
            "AMS" => Some(Self::Ams),
            "PKTLSB" => Some(Self::Pktlsb),
            "PKTUSB" => Some(Self::Pktusb),
            "PKTFM" => Some(Self::Pktfm),
            "ECSSUSB" => Some(Self::Ecssusb),
            "ECSSLSB" => Some(Self::Ecsslsb),
            "FAX" => Some(Self::Fax),
            "SAM" => Some(Self::Sam),
            "DSB" => Some(Self::Dsb),
            "FMN" => Some(Self::Fmn),
            "PKTAM" => Some(Self::Pktam),
            "P25" => Some(Self::P25),
            "DSTAR" => Some(Self::Dstar),
            "DPMR" => Some(Self::Dpmr),
            "NXDNVN" => Some(Self::Nxdnvn),
            "NXDNN" | "NXDN_N" => Some(Self::NxdnN),
            "DCR" => Some(Self::Dcr),
            "AMN" => Some(Self::Amn),
            "PSK" => Some(Self::Psk),
            "PSKR" => Some(Self::Pskr),
            "DD" => Some(Self::Dd),
            "C4FM" => Some(Self::C4fm),
            "PKTFMN" => Some(Self::Pktfmn),
            "SPEC" => Some(Self::Spec),
            "CWN" => Some(Self::Cwn),
            "AM" => Some(Self::Am),
            _ => None,
        }
    }
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
