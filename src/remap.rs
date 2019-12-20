#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::bindings::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_ulong};
use url::{Url, ParseError};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _tsremap_api_info {
    pub size: c_ulong,
    pub tsremap_version: c_ulong,
}
pub type TSRemapInterface = _tsremap_api_info;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _tm_remap_request_info {
    pub mapFromUrl: TSMLoc,
    pub mapToUrl: TSMLoc,
    pub requestUrl: TSMLoc,
    pub requestBufp: TSMBuffer,
    pub requestHdrp: TSMLoc,
    pub redirect: ::std::os::raw::c_int,
}
pub type TSRemapRequestInfo = _tm_remap_request_info;

pub enum TSRemapStatus {
    NoRemap,
    DidRemap,
    NoRemapStop,
    DidRemapStop,
    RemapError,
}

pub type TSHeaders = HashMap<String, Vec<String>>;

pub fn remap_request_url(txn: TSHttpTxn, rri: *mut TSRemapRequestInfo) -> Result<Url, ParseError> {
    if rri.is_null() {
        return Err(ParseError::EmptyHost);
    }

    let url = unsafe {
        let req_info = *rri;

        let mut len: c_int = 0;
        let mut unsafe_url: *mut c_char = TSHttpTxnEffectiveUrlStringGet(txn, &mut len);
        if len == 0 {
            unsafe_url = TSUrlStringGet(req_info.requestBufp, req_info.requestUrl, &mut len);
        }
        if len == 0 {
            return Err(ParseError::EmptyHost);
        }

        CStr::from_ptr(unsafe_url).to_str().map_err(|_e| ParseError::EmptyHost)?
    };

    Url::parse(url)
}

pub fn remap_request_headers(rri: *mut TSRemapRequestInfo) -> Result<TSHeaders, String> {
    if rri.is_null() {
        return Err("remap request info is null".to_string());
    }

    let mut headers = TSHeaders::default();

    unsafe {
        let req_info = *rri;

        let field_len = TSMimeHdrFieldsCount(req_info.requestBufp, req_info.requestHdrp);
        for field_idx in 0..field_len {
            let field = TSMimeHdrFieldGet(req_info.requestBufp, req_info.requestHdrp, field_idx);
            if !field.is_null() {
                let mut len: c_int = 0;
                let field_name: *const c_char = TSMimeHdrFieldNameGet(req_info.requestBufp, req_info.requestHdrp, field, &mut len);
                if field_name.is_null() {
                    continue;
                }

                let name = CStr::from_ptr(field_name).to_str().and_then(|s| Ok(s.to_string())).map_err(|e| e.to_string())?;

                let count = TSMimeHdrFieldValuesCount(req_info.requestBufp, req_info.requestHdrp, field);
                if count > 0 {
                    let mut values = Vec::new();

                    for value_idx in 0..count {
                        let mut len: c_int = 0;
                        let value: *const c_char = TSMimeHdrFieldValueStringGet(req_info.requestBufp, req_info.requestHdrp, field, value_idx, &mut len);

                        if !value.is_null() {
                            let val = CStr::from_ptr(value).to_str().and_then(|s| Ok(s.to_string())).map_err(|e| e.to_string())?;
                            values.push(val);
                        }
                    }

                    headers.insert(name, values);
                }
            }
        }
    };

    Ok(headers)
}