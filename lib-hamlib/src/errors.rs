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
use std::ffi::{c_int, CStr};

#[derive(Debug)]
pub struct HamLibError<'a> {
    pub error_code: u32,
    pub message: &'a str,
}

impl<'a> HamLibError<'a> {
    pub(crate) fn from_hamlib_error_code(error_code: u32) -> HamLibError<'a> {
        let char_ptr = unsafe { crate::hamlib_raw::rigerror(error_code as c_int) };
        let str = unsafe { CStr::from_ptr(char_ptr) }.to_str().unwrap();
        HamLibError {
            error_code,
            message: str,
        }
    }
}
