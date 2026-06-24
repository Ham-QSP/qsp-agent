use crate::errors::HamLibError;
use crate::hamlib_raw;
use crate::hamlib_raw::{
    freq_range_t, rig_caps, rig_debug_level_e, rig_debug_level_e_RIG_DEBUG_BUG,
    rig_debug_level_e_RIG_DEBUG_CACHE, rig_debug_level_e_RIG_DEBUG_ERR,
    rig_debug_level_e_RIG_DEBUG_NONE, rig_debug_level_e_RIG_DEBUG_TRACE,
    rig_debug_level_e_RIG_DEBUG_VERBOSE, rig_debug_level_e_RIG_DEBUG_WARN, rig_errcode_e_RIG_OK,
    rig_load_all_backends, rmode_t, vfo_op_t, RIG_CONF_END, RIG_MODE_NONE,
};
use crate::rig::{Rig, RigVfoOperation};
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_int, c_long};
use std::ptr::null_mut;
use std::sync::{Mutex, OnceLock};

type HamlibToken = c_long;

#[derive(Clone, Debug)]
pub struct RigCaps {
    pub rig_model: u32,
    pub model_name: String,
    pub manufacturer_name: String,
    pub vfo_ops: Vec<RigVfoOperation>,
    pub rx_frequency_ranges: Vec<RigFrequencyRange>,
    pub tx_frequency_ranges: Vec<RigFrequencyRange>,
}

#[derive(Clone, Debug)]
pub struct RigFrequencyRange {
    pub region: u8,
    pub lower_frequency_hz: u64,
    pub upper_frequency_hz: u64,
    pub modes: Vec<RigMode>,
    pub vfo: u32,
    pub antenna: u32,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RigMode {
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

impl RigMode {
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

    fn all() -> &'static [Self] {
        &[
            Self::Cw,
            Self::Usb,
            Self::Lsb,
            Self::Rtty,
            Self::Fm,
            Self::Wfm,
            Self::Cwr,
            Self::Rttyr,
            Self::Ams,
            Self::Pktlsb,
            Self::Pktusb,
            Self::Pktfm,
            Self::Ecssusb,
            Self::Ecsslsb,
            Self::Fax,
            Self::Sam,
            Self::Dsb,
            Self::Fmn,
            Self::Pktam,
            Self::P25,
            Self::Dstar,
            Self::Dpmr,
            Self::Nxdnvn,
            Self::NxdnN,
            Self::Dcr,
            Self::Amn,
            Self::Psk,
            Self::Pskr,
            Self::Dd,
            Self::C4fm,
            Self::Pktfmn,
            Self::Spec,
            Self::Cwn,
            Self::Am,
        ]
    }
}

pub trait RigDebugCallback: Send {
    fn on_debug(&mut self, level: RigDebugLevel, message: &str);
}

impl<F> RigDebugCallback for F
where
    F: FnMut(RigDebugLevel, &str) + Send,
{
    fn on_debug(&mut self, level: RigDebugLevel, message: &str) {
        self(level, message);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RigDebugLevel {
    None,
    Bug,
    Err,
    Warn,
    Verbose,
    Trace,
    Cache,
    Unknown(rig_debug_level_e),
}

impl From<rig_debug_level_e> for RigDebugLevel {
    fn from(level: rig_debug_level_e) -> Self {
        if level == rig_debug_level_e_RIG_DEBUG_NONE {
            Self::None
        } else if level == rig_debug_level_e_RIG_DEBUG_BUG {
            Self::Bug
        } else if level == rig_debug_level_e_RIG_DEBUG_ERR {
            Self::Err
        } else if level == rig_debug_level_e_RIG_DEBUG_WARN {
            Self::Warn
        } else if level == rig_debug_level_e_RIG_DEBUG_VERBOSE {
            Self::Verbose
        } else if level == rig_debug_level_e_RIG_DEBUG_TRACE {
            Self::Trace
        } else if level == rig_debug_level_e_RIG_DEBUG_CACHE {
            Self::Cache
        } else {
            Self::Unknown(level)
        }
    }
}

impl From<RigDebugLevel> for rig_debug_level_e {
    fn from(level: RigDebugLevel) -> Self {
        match level {
            RigDebugLevel::None => rig_debug_level_e_RIG_DEBUG_NONE,
            RigDebugLevel::Bug => rig_debug_level_e_RIG_DEBUG_BUG,
            RigDebugLevel::Err => rig_debug_level_e_RIG_DEBUG_ERR,
            RigDebugLevel::Warn => rig_debug_level_e_RIG_DEBUG_WARN,
            RigDebugLevel::Verbose => rig_debug_level_e_RIG_DEBUG_VERBOSE,
            RigDebugLevel::Trace => rig_debug_level_e_RIG_DEBUG_TRACE,
            RigDebugLevel::Cache => rig_debug_level_e_RIG_DEBUG_CACHE,
            RigDebugLevel::Unknown(level) => level,
        }
    }
}

static DEBUG_CALLBACK: OnceLock<Mutex<Option<Box<dyn RigDebugCallback>>>> = OnceLock::new();

pub type RigListCallbackFn = fn(RigCaps);

unsafe extern "C" fn list_rigs_callback(
    caps: *const rig_caps,
    arg2: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let callback: RigListCallbackFn = core::mem::transmute(arg2);
    callback(rigcaps_mapper(caps));

    1
}

pub(crate) unsafe fn rigcaps_mapper(caps: *const rig_caps) -> RigCaps {
    let model_name = unsafe { CStr::from_ptr((*caps).model_name) };
    let model_name = model_name.to_str().unwrap().to_string();

    let manufacturer_name = unsafe { CStr::from_ptr((*caps).mfg_name) };
    let manufacturer_name = manufacturer_name.to_str().unwrap().to_string();
    let rig = RigCaps {
        rig_model: (*caps).rig_model,
        model_name,
        manufacturer_name,
        vfo_ops: vfo_ops_mapper((*caps).vfo_ops),
        rx_frequency_ranges: freq_ranges_mapper(&[
            (1, &(*caps).rx_range_list1),
            (2, &(*caps).rx_range_list2),
            (3, &(*caps).rx_range_list3),
            (4, &(*caps).rx_range_list4),
            (5, &(*caps).rx_range_list5),
        ]),
        tx_frequency_ranges: freq_ranges_mapper(&[
            (1, &(*caps).tx_range_list1),
            (2, &(*caps).tx_range_list2),
            (3, &(*caps).tx_range_list3),
            (4, &(*caps).tx_range_list4),
            (5, &(*caps).tx_range_list5),
        ]),
    };
    rig
}

fn vfo_ops_mapper(vfo_ops: vfo_op_t) -> Vec<RigVfoOperation> {
    RigVfoOperation::all()
        .iter()
        .filter_map(|(operation, bit)| {
            if vfo_ops & *bit == *bit {
                Some(*operation)
            } else {
                None
            }
        })
        .collect()
}

fn freq_ranges_mapper(range_lists: &[(u8, &[freq_range_t; 30])]) -> Vec<RigFrequencyRange> {
    range_lists
        .iter()
        .flat_map(|(region, ranges)| {
            ranges
                .iter()
                .take_while(|range| range.startf != 0.0 || range.endf != 0.0)
                .map(|range| RigFrequencyRange {
                    region: *region,
                    lower_frequency_hz: range.startf as u64,
                    upper_frequency_hz: range.endf as u64,
                    modes: modes_mapper(range.modes),
                    vfo: range.vfo,
                    antenna: range.ant,
                    label: label_mapper(range.label),
                })
        })
        .collect()
}

fn modes_mapper(modes: rmode_t) -> Vec<RigMode> {
    parsed_modes()
        .iter()
        .filter_map(|(mode, bit)| {
            if modes & *bit == *bit {
                Some(*mode)
            } else {
                None
            }
        })
        .collect()
}

fn parsed_modes() -> &'static [(RigMode, rmode_t)] {
    static PARSED_MODES: OnceLock<Vec<(RigMode, rmode_t)>> = OnceLock::new();

    PARSED_MODES.get_or_init(|| {
        RigMode::all()
            .iter()
            .filter_map(|mode| {
                let mode_name = CString::new(mode.as_hamlib_name()).ok()?;
                let bit = unsafe { hamlib_raw::rig_parse_mode(mode_name.as_ptr()) };
                if bit == RIG_MODE_NONE as rmode_t {
                    None
                } else {
                    Some((*mode, bit))
                }
            })
            .collect()
    })
}

fn label_mapper(label: *mut ::std::os::raw::c_char) -> Option<String> {
    if label.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(label) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

unsafe extern "C" fn hamlib_debug_callback_trampoline(
    level: rig_debug_level_e,
    _arg: *mut c_void,
    fmt: *const ::std::os::raw::c_char,
    ap: hamlib_raw::va_list,
) -> c_int {
    let mut rendered = null_mut();
    let formatted_len = unsafe { vasprintf_with_va_list(&mut rendered, fmt, ap) };

    if formatted_len < 0 || rendered.is_null() {
        return formatted_len;
    }

    let message = unsafe { CStr::from_ptr(rendered) }
        .to_string_lossy()
        .into_owned();
    unsafe { free(rendered.cast()) };

    if let Some(callback_slot) = DEBUG_CALLBACK.get() {
        if let Some(callback) = callback_slot.lock().unwrap().as_mut() {
            callback.on_debug(level.into(), &message);
        }
    }

    formatted_len
}

#[cfg(target_os = "linux")]
unsafe fn vasprintf_with_va_list(
    rendered: *mut *mut ::std::os::raw::c_char,
    fmt: *const ::std::os::raw::c_char,
    mut ap: hamlib_raw::va_list,
) -> c_int {
    unsafe { hamlib_raw::vasprintf(rendered, fmt, ap.as_mut_ptr()) }
}

#[cfg(not(target_os = "linux"))]
unsafe fn vasprintf_with_va_list(
    rendered: *mut *mut ::std::os::raw::c_char,
    fmt: *const ::std::os::raw::c_char,
    ap: hamlib_raw::va_list,
) -> c_int {
    unsafe { hamlib_raw::vasprintf(rendered, fmt, ap) }
}

unsafe extern "C" {
    fn free(ptr: *mut c_void);
}

pub struct Hamlib {
    pub(crate) all_backends_loaded: bool,
}

impl Hamlib {
    pub fn new() -> Hamlib {
        Hamlib {
            all_backends_loaded: false,
        }
    }

    pub fn rig_set_debug_callback(callback: Option<Box<dyn RigDebugCallback>>) {
        let callback_slot = DEBUG_CALLBACK.get_or_init(|| Mutex::new(None));
        let has_callback = callback.is_some();
        *callback_slot.lock().unwrap() = callback;

        unsafe {
            hamlib_raw::rig_set_debug_callback(
                if has_callback {
                    Some(hamlib_debug_callback_trampoline)
                } else {
                    None
                },
                null_mut(),
            );
        }
    }

    pub fn rig_set_debug(level: RigDebugLevel) {
        unsafe {
            hamlib_raw::rig_set_debug(level.into());
        }
    }

    pub fn list_rigs(&mut self, callback: RigListCallbackFn) {
        if !self.all_backends_loaded {
            unsafe {
                rig_load_all_backends();
            }
            self.all_backends_loaded = true;
        }
        unsafe {
            hamlib_raw::rig_list_foreach(Some(list_rigs_callback), core::mem::transmute(callback));
        }
    }

    pub fn rig_connect(
        &mut self,
        rig_model: u32,
        config: HashMap<String, String>,
    ) -> Result<Rig, HamLibError<'_>> {
        unsafe {
            let rig = hamlib_raw::rig_init(rig_model);
            for (key, value) in config {
                let token = rig_token_lookup(rig, &key)?;
                rig_set_conf(rig, token, &value)?;
            }

            let open_result = hamlib_raw::rig_open(rig) as u32;
            if open_result == rig_errcode_e_RIG_OK {
                return Ok(Rig { rig });
            }
            Err(HamLibError::from_hamlib_error_code(open_result))
        }
    }
}

unsafe fn rig_token_lookup<'a>(
    rig: *mut hamlib_raw::RIG,
    name: &str,
) -> Result<HamlibToken, HamLibError<'a>> {
    let name = CString::new(name).unwrap();
    let token = unsafe { hamlib_raw::rig_token_lookup(rig, name.as_ptr()) };
    if token == RIG_CONF_END as HamlibToken {
        return Err(HamLibError {
            error_code: RIG_CONF_END,
            message: "unknown hamlib config token",
        });
    }

    Ok(token)
}

unsafe fn rig_set_conf<'a>(
    rig: *mut hamlib_raw::RIG,
    token: HamlibToken,
    value: &str,
) -> Result<(), HamLibError<'a>> {
    let value = CString::new(value).unwrap();
    let result = unsafe { hamlib_raw::rig_set_conf(rig, token, value.as_ptr()) as u32 };

    if result == rig_errcode_e_RIG_OK {
        Ok(())
    } else {
        Err(HamLibError::from_hamlib_error_code(result))
    }
}

#[cfg(test)]
mod tests {
    use super::vfo_ops_mapper;
    use crate::hamlib_raw::{vfo_op_t_RIG_OP_BAND_UP, vfo_op_t_RIG_OP_CPY, vfo_op_t_RIG_OP_TUNE};
    use crate::rig::RigVfoOperation;

    #[test]
    fn maps_vfo_ops_bitfield_to_enum_list() {
        let mapped =
            vfo_ops_mapper(vfo_op_t_RIG_OP_CPY | vfo_op_t_RIG_OP_BAND_UP | vfo_op_t_RIG_OP_TUNE);

        assert_eq!(
            mapped,
            vec![
                RigVfoOperation::Copy,
                RigVfoOperation::BandUp,
                RigVfoOperation::Tune
            ]
        );
    }
}
