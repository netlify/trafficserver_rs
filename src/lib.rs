
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};

pub mod bindings;
pub use bindings::*;

mod remap;
pub use remap::*;

pub fn ts_debug(tag: &str, message: &str) {
    let t = CString::new(tag).unwrap_or_default();
    let s = CString::new(message).unwrap_or_default();
    unsafe { TSDebug(t.as_ptr(), s.as_ptr()); }
}

pub fn ts_error(message: &str) {
    let s = CString::new(message).unwrap_or_default();
    unsafe { TSError(s.as_ptr()); }
}

pub fn ts_config_dir_get() -> &'static str {
    let dir = unsafe { 
        let dir = TSConfigDirGet();
        if dir.is_null() { Default::default() } else { CStr::from_ptr(dir) }
    };

    dir.to_str().unwrap_or_default()
}