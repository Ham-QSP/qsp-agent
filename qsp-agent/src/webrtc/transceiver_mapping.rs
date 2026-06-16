/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>
 */
use crate::hardware::transceiver::transceiver_state::{TransceiverBand, TransceiverMode};
use qsp_proto_files::qsp::message::v1::{Band, TrxVfoMode};

pub(super) fn trx_vfo_mode_to_transceiver_mode(mode: TrxVfoMode) -> Option<TransceiverMode> {
    match mode {
        TrxVfoMode::Unspecified => None,
        TrxVfoMode::Cw => Some(TransceiverMode::Cw),
        TrxVfoMode::Usb => Some(TransceiverMode::Usb),
        TrxVfoMode::Lsb => Some(TransceiverMode::Lsb),
        TrxVfoMode::Rtty => Some(TransceiverMode::Rtty),
        TrxVfoMode::Fm => Some(TransceiverMode::Fm),
        TrxVfoMode::Wfm => Some(TransceiverMode::Wfm),
        TrxVfoMode::Cwr => Some(TransceiverMode::Cwr),
        TrxVfoMode::Rttyr => Some(TransceiverMode::Rttyr),
        TrxVfoMode::Ams => Some(TransceiverMode::Ams),
        TrxVfoMode::Pktlsb => Some(TransceiverMode::Pktlsb),
        TrxVfoMode::Pktusb => Some(TransceiverMode::Pktusb),
        TrxVfoMode::Pktfm => Some(TransceiverMode::Pktfm),
        TrxVfoMode::Ecssusb => Some(TransceiverMode::Ecssusb),
        TrxVfoMode::Exsslsb => Some(TransceiverMode::Ecsslsb),
        TrxVfoMode::Fax => Some(TransceiverMode::Fax),
        TrxVfoMode::Sam => Some(TransceiverMode::Sam),
        TrxVfoMode::Dsb => Some(TransceiverMode::Dsb),
        TrxVfoMode::Fmn => Some(TransceiverMode::Fmn),
        TrxVfoMode::Pktam => Some(TransceiverMode::Pktam),
        TrxVfoMode::P25 => Some(TransceiverMode::P25),
        TrxVfoMode::Dstar => Some(TransceiverMode::Dstar),
        TrxVfoMode::Dpmr => Some(TransceiverMode::Dpmr),
        TrxVfoMode::Nxdnvn => Some(TransceiverMode::Nxdnvn),
        TrxVfoMode::NxdnN => Some(TransceiverMode::NxdnN),
        TrxVfoMode::Dcr => Some(TransceiverMode::Dcr),
        TrxVfoMode::Amn => Some(TransceiverMode::Amn),
        TrxVfoMode::Psk => Some(TransceiverMode::Psk),
        TrxVfoMode::Pskr => Some(TransceiverMode::Pskr),
        TrxVfoMode::Dd => Some(TransceiverMode::Dd),
        TrxVfoMode::C4fm => Some(TransceiverMode::C4fm),
        TrxVfoMode::Pktfmn => Some(TransceiverMode::Pktfmn),
        TrxVfoMode::Spec => Some(TransceiverMode::Spec),
        TrxVfoMode::Cwn => Some(TransceiverMode::Cwn),
        TrxVfoMode::Am => Some(TransceiverMode::Am),
    }
}

pub(super) fn transceiver_mode_to_trx_vfo_mode(mode: TransceiverMode) -> TrxVfoMode {
    match mode {
        TransceiverMode::Cw => TrxVfoMode::Cw,
        TransceiverMode::Usb => TrxVfoMode::Usb,
        TransceiverMode::Lsb => TrxVfoMode::Lsb,
        TransceiverMode::Rtty => TrxVfoMode::Rtty,
        TransceiverMode::Fm => TrxVfoMode::Fm,
        TransceiverMode::Wfm => TrxVfoMode::Wfm,
        TransceiverMode::Cwr => TrxVfoMode::Cwr,
        TransceiverMode::Rttyr => TrxVfoMode::Rttyr,
        TransceiverMode::Ams => TrxVfoMode::Ams,
        TransceiverMode::Pktlsb => TrxVfoMode::Pktlsb,
        TransceiverMode::Pktusb => TrxVfoMode::Pktusb,
        TransceiverMode::Pktfm => TrxVfoMode::Pktfm,
        TransceiverMode::Ecssusb => TrxVfoMode::Ecssusb,
        TransceiverMode::Ecsslsb => TrxVfoMode::Exsslsb,
        TransceiverMode::Fax => TrxVfoMode::Fax,
        TransceiverMode::Sam => TrxVfoMode::Sam,
        TransceiverMode::Dsb => TrxVfoMode::Dsb,
        TransceiverMode::Fmn => TrxVfoMode::Fmn,
        TransceiverMode::Pktam => TrxVfoMode::Pktam,
        TransceiverMode::P25 => TrxVfoMode::P25,
        TransceiverMode::Dstar => TrxVfoMode::Dstar,
        TransceiverMode::Dpmr => TrxVfoMode::Dpmr,
        TransceiverMode::Nxdnvn => TrxVfoMode::Nxdnvn,
        TransceiverMode::NxdnN => TrxVfoMode::NxdnN,
        TransceiverMode::Dcr => TrxVfoMode::Dcr,
        TransceiverMode::Amn => TrxVfoMode::Amn,
        TransceiverMode::Psk => TrxVfoMode::Psk,
        TransceiverMode::Pskr => TrxVfoMode::Pskr,
        TransceiverMode::Dd => TrxVfoMode::Dd,
        TransceiverMode::C4fm => TrxVfoMode::C4fm,
        TransceiverMode::Pktfmn => TrxVfoMode::Pktfmn,
        TransceiverMode::Spec => TrxVfoMode::Spec,
        TransceiverMode::Cwn => TrxVfoMode::Cwn,
        TransceiverMode::Am => TrxVfoMode::Am,
    }
}

pub(super) fn band_to_transceiver_band(band: Band) -> Option<TransceiverBand> {
    match band {
        Band::Unspecified => None,
        Band::Band2200m => Some(TransceiverBand::Band2200m),
        Band::Band600m => Some(TransceiverBand::Band600m),
        Band::Band160m => Some(TransceiverBand::Band160m),
        Band::Band80m => Some(TransceiverBand::Band80m),
        Band::Band60m => Some(TransceiverBand::Band60m),
        Band::Band40m => Some(TransceiverBand::Band40m),
        Band::Band30m => Some(TransceiverBand::Band30m),
        Band::Band20m => Some(TransceiverBand::Band20m),
        Band::Band17m => Some(TransceiverBand::Band17m),
        Band::Band15m => Some(TransceiverBand::Band15m),
        Band::Band12m => Some(TransceiverBand::Band12m),
        Band::Band1om => Some(TransceiverBand::Band10m),
        Band::Band6m => Some(TransceiverBand::Band6m),
        Band::Band4m => Some(TransceiverBand::Band4m),
        Band::Band2m => Some(TransceiverBand::Band2m),
        Band::Band125m => Some(TransceiverBand::Band125m),
        Band::Band70cm => Some(TransceiverBand::Band70cm),
        Band::Band33cm => Some(TransceiverBand::Band33cm),
        Band::Band23cm => Some(TransceiverBand::Band23cm),
        Band::Band13cm => Some(TransceiverBand::Band13cm),
        Band::Band9cm => Some(TransceiverBand::Band9cm),
        Band::Band5cm => Some(TransceiverBand::Band5cm),
        Band::Band3cm => Some(TransceiverBand::Band3cm),
        Band::Band12mm => Some(TransceiverBand::Band12mm),
    }
}
