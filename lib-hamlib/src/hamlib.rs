use std::ffi::{c_void, CStr};
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::{Mutex, OnceLock};
use crate::errors::HamLibError;
use crate::{hamlib, hamlib_raw, rig};
use crate::hamlib_raw::{rig_caps, rig_debug_level_e, rig_debug_level_e_RIG_DEBUG_BUG, rig_debug_level_e_RIG_DEBUG_CACHE, rig_debug_level_e_RIG_DEBUG_ERR, rig_debug_level_e_RIG_DEBUG_NONE, rig_debug_level_e_RIG_DEBUG_TRACE, rig_debug_level_e_RIG_DEBUG_VERBOSE, rig_debug_level_e_RIG_DEBUG_WARN, rig_errcode_e_RIG_OK, rig_load_all_backends};
use crate::rig::Rig;

pub struct RigCaps {
    pub rig_model: u32,
    pub model_name: String,
    pub manufacturer_name: String,
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

unsafe extern "C" fn hamlib_debug_callback_trampoline(
    level: rig_debug_level_e,
    _arg: *mut c_void,
    fmt: *const ::std::os::raw::c_char,
    ap: hamlib_raw::va_list,
) -> c_int {
    let mut rendered = null_mut();
    let formatted_len = unsafe { hamlib_raw::vasprintf(&mut rendered, fmt, ap) };

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

    pub fn list_rigs(&mut self, callback: RigListCallbackFn) {
        if !self.all_backends_loaded {
            unsafe {
                rig_load_all_backends();
            }
            self.all_backends_loaded = true;
        }
        unsafe {
            hamlib_raw::rig_list_foreach(
                Some(list_rigs_callback),
                core::mem::transmute(callback),
            );
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