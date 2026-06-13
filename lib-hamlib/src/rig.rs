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
use crate::errors::HamLibError;
use crate::hamlib::{rigcaps_mapper, RigCaps};
use crate::hamlib_raw;
use crate::hamlib_raw::{
    freq_t, hamlib_bandselect_t_RIG_BANDSELECT_10M, hamlib_bandselect_t_RIG_BANDSELECT_12M,
    hamlib_bandselect_t_RIG_BANDSELECT_13CM, hamlib_bandselect_t_RIG_BANDSELECT_15M,
    hamlib_bandselect_t_RIG_BANDSELECT_160M, hamlib_bandselect_t_RIG_BANDSELECT_17M,
    hamlib_bandselect_t_RIG_BANDSELECT_1_25M, hamlib_bandselect_t_RIG_BANDSELECT_20M,
    hamlib_bandselect_t_RIG_BANDSELECT_2200M, hamlib_bandselect_t_RIG_BANDSELECT_23CM,
    hamlib_bandselect_t_RIG_BANDSELECT_2M, hamlib_bandselect_t_RIG_BANDSELECT_30M,
    hamlib_bandselect_t_RIG_BANDSELECT_33CM, hamlib_bandselect_t_RIG_BANDSELECT_3CM,
    hamlib_bandselect_t_RIG_BANDSELECT_40M, hamlib_bandselect_t_RIG_BANDSELECT_4M,
    hamlib_bandselect_t_RIG_BANDSELECT_5CM, hamlib_bandselect_t_RIG_BANDSELECT_600M,
    hamlib_bandselect_t_RIG_BANDSELECT_60M, hamlib_bandselect_t_RIG_BANDSELECT_6M,
    hamlib_bandselect_t_RIG_BANDSELECT_70CM, hamlib_bandselect_t_RIG_BANDSELECT_80M,
    hamlib_bandselect_t_RIG_BANDSELECT_9CM, hamlib_bandselect_t_RIG_BANDSELECT_AIR,
    hamlib_bandselect_t_RIG_BANDSELECT_GEN, hamlib_bandselect_t_RIG_BANDSELECT_MW,
    hamlib_bandselect_t_RIG_BANDSELECT_WFM, pbwidth_t, rig_errcode_e_RIG_OK,
    rig_parm_e_RIG_PARM_BANDSELECT, rmode_t, value_t, vfo_op_t, vfo_op_t_RIG_OP_BAND_DOWN,
    vfo_op_t_RIG_OP_BAND_UP, vfo_t, RIG, RIG_MODE_NONE,
};
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint};
pub struct CCallback<'closure> {
    pub function: unsafe extern "C" fn(
        arg1: *mut RIG,
        arg2: vfo_t,
        arg3: freq_t,
        arg4: *mut ::std::os::raw::c_void,
    ) -> c_int,
    pub user_data: *mut c_void,

    _lifetime: PhantomData<&'closure mut c_void>,
}

impl<'closure> CCallback<'closure> {
    pub fn new<F>(closure: &'closure mut F) -> Self
    where
        F: FnMut(f64, c_uint) -> c_int,
    {
        let function: unsafe extern "C" fn(
            arg1: *mut RIG,
            arg2: vfo_t,
            arg3: freq_t,
            user_data: *mut ::std::os::raw::c_void,
        ) -> c_int = Self::call_closure::<F>;

        debug_assert_eq!(
            std::mem::size_of::<&'closure mut F>(),
            std::mem::size_of::<*const c_void>()
        );
        debug_assert_eq!(
            std::mem::size_of_val(&function),
            std::mem::size_of::<*const c_void>()
        );

        Self {
            function,
            user_data: closure as *mut F as *mut c_void,
            _lifetime: PhantomData,
        }
    }

    unsafe extern "C" fn call_closure<F>(
        rig: *mut RIG,
        vfo: vfo_t,
        freq: freq_t,
        user_data: *mut ::std::os::raw::c_void,
    ) -> c_int
    where
        F: FnMut(f64, c_uint) -> c_int,
    {
        let cb: &mut F = user_data.cast::<F>().as_mut().unwrap();
        (*cb)(freq, vfo)
    }
}
pub struct Rig {
    pub(crate) rig: *mut RIG,
}

#[derive(Clone, Copy, Debug)]
pub enum RigVfoOperation {
    BandUp,
    BandDown,
}

impl RigVfoOperation {
    fn as_hamlib_vfo_op(self) -> vfo_op_t {
        match self {
            Self::BandUp => vfo_op_t_RIG_OP_BAND_UP,
            Self::BandDown => vfo_op_t_RIG_OP_BAND_DOWN,
        }
    }
}

// SAFETY: Rig owns an opaque hamlib handle. Callers that share it across
// threads must provide synchronization around hamlib calls.
unsafe impl Send for Rig {}

pub type RigFreqCallback = fn();

impl Rig {
    pub fn caps(&self) -> Option<RigCaps> {
        unsafe {
            let caps = (*self.rig).caps;
            if caps.is_null() {
                None
            } else {
                Some(rigcaps_mapper(caps))
            }
        }
    }

    pub fn set_freq_callback<F>(&self, closure: &mut F)
    where
        F: FnMut(f64, c_uint) -> c_int,
    {
        //let closure = &mut |x: f64, y: c_uint| { println!("CALLBACK: {:?}", x); 1};
        let c = CCallback::new(closure);

        unsafe {
            hamlib_raw::rig_set_freq_callback(self.rig, Some(c.function), c.user_data);
        }
    }

    pub fn set_freq(&self, vfo: u32, freq: freq_t) {
        unsafe {
            hamlib_raw::rig_set_freq(self.rig, vfo, freq);
        }
    }

    pub fn set_mode(&self, vfo: u32, mode: &str) -> Result<(), HamLibError<'_>> {
        let mode = CString::new(mode).map_err(|_| HamLibError {
            error_code: 0,
            message: "mode contains an interior null byte",
        })?;

        unsafe {
            let mode: rmode_t = hamlib_raw::rig_parse_mode(mode.as_ptr());
            if mode == RIG_MODE_NONE as rmode_t {
                return Err(HamLibError {
                    error_code: 0,
                    message: "unknown hamlib mode",
                });
            }

            let width: pbwidth_t = hamlib_raw::rig_passband_normal(self.rig, mode);
            let ret = hamlib_raw::rig_set_mode(self.rig, vfo, mode, width) as u32;
            if ret == rig_errcode_e_RIG_OK {
                Ok(())
            } else {
                Err(HamLibError::from_hamlib_error_code(ret))
            }
        }
    }

    pub fn set_band_select(&self, band: u32) -> Result<(), HamLibError<'_>> {
        unsafe {
            let value = value_t { i: band as i32 };
            let ret =
                hamlib_raw::rig_set_parm(self.rig, rig_parm_e_RIG_PARM_BANDSELECT as u64, value)
                    as u32;
            if ret == rig_errcode_e_RIG_OK {
                Ok(())
            } else {
                Err(HamLibError::from_hamlib_error_code(ret))
            }
        }
    }

    pub fn set_band(&self, band: &str) -> Result<(), HamLibError<'_>> {
        let band = parse_band_select(band).ok_or(HamLibError {
            error_code: 0,
            message: "unknown hamlib band",
        })?;
        self.set_band_select(band)
    }

    pub fn vfo_op(&self, vfo: u32, operation: RigVfoOperation) -> Result<(), HamLibError<'_>> {
        unsafe {
            let ret = hamlib_raw::rig_vfo_op(self.rig, vfo, operation.as_hamlib_vfo_op()) as u32;
            if ret == rig_errcode_e_RIG_OK {
                Ok(())
            } else {
                Err(HamLibError::from_hamlib_error_code(ret))
            }
        }
    }

    pub fn get_freq(&self, vfo: u32) -> Result<freq_t, HamLibError<'_>> {
        unsafe {
            let mut freq: freq_t = 0.0;
            let freq_ptr: *mut freq_t = &mut freq;

            let ret = hamlib_raw::rig_get_freq(self.rig, vfo, freq_ptr) as u32;
            if ret == rig_errcode_e_RIG_OK {
                return Ok(freq);
            }
            return Err(HamLibError::from_hamlib_error_code(ret));
        }
    }

    pub fn get_mode(&self, vfo: u32) -> Result<String, HamLibError<'_>> {
        unsafe {
            let mut mode: rmode_t = RIG_MODE_NONE as rmode_t;
            let mut width: pbwidth_t = 0;

            let ret = hamlib_raw::rig_get_mode(self.rig, vfo, &mut mode, &mut width) as u32;
            if ret == rig_errcode_e_RIG_OK {
                let mode = CStr::from_ptr(hamlib_raw::rig_strrmode(mode))
                    .to_string_lossy()
                    .into_owned();
                return Ok(mode);
            }
            Err(HamLibError::from_hamlib_error_code(ret))
        }
    }
}

fn parse_band_select(band: &str) -> Option<u32> {
    let normalized = band
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "");

    match normalized.as_str() {
        "2200m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_2200M),
        "600m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_600M),
        "160m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_160M),
        "80m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_80M),
        "60m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_60M),
        "40m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_40M),
        "30m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_30M),
        "20m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_20M),
        "17m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_17M),
        "15m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_15M),
        "12m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_12M),
        "10m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_10M),
        "6m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_6M),
        "4m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_4M),
        "2m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_2M),
        "125m" | "1.25m" => Some(hamlib_bandselect_t_RIG_BANDSELECT_1_25M),
        "70cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_70CM),
        "33cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_33CM),
        "23cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_23CM),
        "13cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_13CM),
        "9cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_9CM),
        "5cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_5CM),
        "3cm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_3CM),
        "wfm" => Some(hamlib_bandselect_t_RIG_BANDSELECT_WFM),
        "gen" | "general" => Some(hamlib_bandselect_t_RIG_BANDSELECT_GEN),
        "mw" => Some(hamlib_bandselect_t_RIG_BANDSELECT_MW),
        "air" => Some(hamlib_bandselect_t_RIG_BANDSELECT_AIR),
        _ => None,
    }
}
