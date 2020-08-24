#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};

pub mod bindings;
pub use bindings::*;

pub mod cache;
pub mod continuations;
pub mod headers;
mod helpers;
pub mod remap;
pub mod request;
pub mod response;
pub mod status;
pub mod transaction;
pub mod url;
pub use remap::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Request(#[from] request::Error),

    #[error(transparent)]
    Response(#[from] response::Error),

    #[error(transparent)]
    Transaction(#[from] transaction::Error),

    #[error(transparent)]
    InvalidRawString(#[from] crate::helpers::RawStrError),
}

pub fn ts_debug(tag: &str, message: &str) {
    let t = CString::new(tag).unwrap_or_default();
    let s = CString::new(message).unwrap_or_default();
    unsafe {
        TSDebug(t.as_ptr(), s.as_ptr());
    }
}

pub fn ts_error(message: &str) {
    let s = CString::new(message).unwrap_or_default();
    unsafe {
        TSError(s.as_ptr());
    }
}

pub fn ts_config_dir_get() -> &'static str {
    let dir = unsafe {
        let dir = TSConfigDirGet();
        if dir.is_null() {
            Default::default()
        } else {
            CStr::from_ptr(dir)
        }
    };

    dir.to_str().unwrap_or_default()
}
