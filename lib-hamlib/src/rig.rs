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
use crate::hamlib_raw;
use crate::hamlib_raw::{
    freq_t, rig_caps, rig_errcode_e_RIG_OK, rig_load_all_backends, vfo_t, RIG,
};
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint};

pub struct Hamlib {
    all_backends_loaded: bool,
}

pub struct RigCaps {
    pub rig_model: u32,
    pub model_name: String,
    pub manufacturer_name: String,
}

pub struct CCallback<'closure> {
    pub function: unsafe extern "C" fn (
    arg1: *mut RIG,
    arg2: vfo_t,
    arg3: freq_t,
    arg4: *mut ::std::os::raw::c_void,
    ) -> c_int,
    pub user_data: *mut c_void,

    _lifetime: PhantomData<&'closure mut c_void>,
}

impl<'closure> CCallback<'closure> {
    pub fn new<F>(closure: &'closure mut F) -> Self where F: FnMut(f64,c_uint) -> c_int {
        let function: unsafe extern "C" fn (
            arg1: *mut RIG,
            arg2: vfo_t,
            arg3: freq_t,
            user_data: *mut ::std::os::raw::c_void,
        ) -> c_int = Self::call_closure::<F>;

        debug_assert_eq!(std::mem::size_of::<&'closure mut F>(), std::mem::size_of::<*const c_void>());
        debug_assert_eq!(std::mem::size_of_val(&function), std::mem::size_of::<*const c_void>());

        Self {
            function,
            user_data: closure as *mut F as *mut c_void,
            _lifetime: PhantomData,
        }
    }

    unsafe extern "C" fn call_closure<F>(rig: *mut RIG,
                                         vfo: vfo_t,
                                         freq: freq_t,
                                         user_data: *mut ::std::os::raw::c_void,) -> c_int where F: FnMut(f64,c_uint) -> c_int {
        let cb: &mut F = user_data.cast::<F>().as_mut().unwrap();
        (*cb)(freq, vfo)
    }
}
pub struct Rig {
    rig: *mut RIG,
}

pub type RigFreqCallback = fn();

impl Rig {

    pub fn set_freq_callback<F>(&self,closure: &mut F ) where F: FnMut(f64,c_uint) -> c_int {
        //let closure = &mut |x: f64, y: c_uint| { println!("CALLBACK: {:?}", x); 1};
        let c = CCallback::new(closure);

        unsafe {
            hamlib_raw::rig_set_freq_callback(
                self.rig,
                Some(c.function),
                c.user_data);
        }
    }
    
    pub fn set_freq(&self, vfo: u32, freq: freq_t) {
        unsafe {
            hamlib_raw::rig_set_freq(
                self.rig,
                vfo,
                freq
            );
        }
    }
    
    pub fn get_freq(&self, vfo: u32) -> Result<freq_t, HamLibError> {
        unsafe {
            let mut freq: freq_t = 0.0;
            let freq_ptr: *mut freq_t = &mut freq;

            let ret = hamlib_raw::rig_get_freq(self.rig, vfo, freq_ptr) as u32;
            if ret == rig_errcode_e_RIG_OK {
                return Ok(freq);
            }
            return
                Err(HamLibError::from_hamlib_error_code(ret))   
        }
    }
    
}

pub type RigListCallbackFn = fn(RigCaps);

impl Hamlib {
    pub fn new() -> Hamlib {
        Hamlib {
            all_backends_loaded: false,
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
    
    

    pub fn rig_connect(&mut self, rig_model: u32) -> Result<Rig, HamLibError> {
        unsafe {
            let rig = hamlib_raw::rig_init(rig_model);
            let open_result = hamlib_raw::rig_open(rig) as u32;
            if open_result == rig_errcode_e_RIG_OK {
                return Ok(Rig { rig });
            }
            Err(HamLibError::from_hamlib_error_code(open_result))
        }
    }
}

unsafe extern "C" fn list_rigs_callback(
    caps: *const rig_caps,
    arg2: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let callback: RigListCallbackFn = core::mem::transmute(arg2);
    callback(rigcaps_mapper(caps));

    1
}

unsafe fn rigcaps_mapper(caps: *const rig_caps) -> RigCaps {
    let model_name = unsafe { CStr::from_ptr((*caps).model_name) };
    let model_name = model_name.to_str().unwrap().to_string();

    let manufacturer_name = unsafe { CStr::from_ptr((*caps).mfg_name) };
    let manufacturer_name = manufacturer_name.to_str().unwrap().to_string();
    let rig = RigCaps {
        rig_model: (*caps).rig_model,
        model_name,
        manufacturer_name,
    };
    rig
}

unsafe extern "C" fn rig_set_freq_callback(
    arg1: *mut RIG,
    arg2: vfo_t,
    arg3: freq_t,
    arg4: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let callback: RigFreqCallback = core::mem::transmute(arg4);
    callback();

    1
}
