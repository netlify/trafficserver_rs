use crate::bindings::*;

use std::{convert::TryInto, ffi::c_void};

pub fn ts_result_convert(ret: TSReturnCode) -> Result<(), ()> {
    if ret == TSReturnCode_TS_SUCCESS {
        Ok(())
    } else {
        Err(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RawStrError {
    #[error("invalid char pointer")]
    InvalidStringPointer,

    #[error("invalid string length: {0}")]
    InvalidStringLength(std::num::TryFromIntError),

    #[error("invalid UTF8: {0}")]
    InvalidString(std::str::Utf8Error),
}

pub fn checked_raw_str<'a, C>(cb: C) -> Result<&'a str, RawStrError>
where
    C: Fn(&mut i32) -> *const i8,
{
    let mut len: i32 = 0;
    let c_str = cb(&mut len);
    if c_str.is_null() {
        return Err(RawStrError::InvalidStringPointer);
    }
    let len: usize = len.try_into().map_err(RawStrError::InvalidStringLength)?;

    let slice = unsafe { std::slice::from_raw_parts(c_str as *const u8, len) };
    std::str::from_utf8(slice).map_err(RawStrError::InvalidString)
}

pub struct RawString(*mut i8, usize);

// todo(marcus): Define TSFree properly
extern "C" {
    pub fn _TSfree(ptr: *mut c_void);
}

impl Drop for RawString {
    fn drop(&mut self) {
        unsafe { _TSfree(self.0 as *mut c_void) };
    }
}

impl std::ops::Deref for RawString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let slice = unsafe { std::slice::from_raw_parts(self.0 as *const u8, self.1) };
        // SAFETY: only constructed from `checked_raw_string` where it's checked
        unsafe { std::str::from_utf8_unchecked(slice) }
    }
}

pub fn checked_raw_string<C>(cb: C) -> Result<RawString, RawStrError>
where
    C: Fn(&mut i32) -> *mut i8,
{
    let mut len: i32 = 0;
    let c_str = cb(&mut len);
    if c_str.is_null() {
        return Err(RawStrError::InvalidStringPointer);
    }

    let len: usize = len.try_into().map_err(RawStrError::InvalidStringLength)?;

    // verify utf-8 content
    let slice = unsafe { std::slice::from_raw_parts(c_str as *const u8, len) };
    std::str::from_utf8(slice).map_err(RawStrError::InvalidString)?;

    Ok(RawString(c_str, len))
}
