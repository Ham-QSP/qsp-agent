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
use crate::hamlib_raw::{
    freq_t


    , rig_errcode_e_RIG_OK
    , vfo_t, RIG,
};
use crate::hamlib_raw;
use std::ffi::c_void;
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

pub type RigFreqCallback = fn();

impl Rig {
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
}
