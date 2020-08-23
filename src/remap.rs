#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::{bindings::*, request::Request, url::Url};

use std::os::raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _tsremap_api_info {
    pub size: c_ulong,
    pub tsremap_version: c_ulong,
}
pub type TSRemapInterface = _tsremap_api_info;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct _tm_remap_request_info {
    pub mapFromUrl: TSMLoc,
    pub mapToUrl: TSMLoc,
    pub requestUrl: TSMLoc,
    pub requestBufp: TSMBuffer,
    pub requestHdrp: TSMLoc,
    pub redirect: ::std::os::raw::c_int,
}
pub type TSRemapRequestInfo = *mut _tm_remap_request_info;

pub struct RemapRequestInfo<'a> {
    raw: &'a mut _tm_remap_request_info,
}

impl<'a> RemapRequestInfo<'a> {
    /// Wrap a remap request info struct
    ///
    /// # Safety
    /// Dereferences the raw pointer.
    /// Do not pass a null-pointer.
    pub unsafe fn new(raw: TSRemapRequestInfo) -> Self {
        let deref = &mut *raw;
        Self { raw: deref }
    }

    pub fn get_mapped_from(&'a self) -> Url<'a, Self> {
        Url::new(self.raw.requestBufp, self.raw.mapFromUrl)
    }

    pub fn get_mapped_to(&'a self) -> Url<'a, Self> {
        Url::new(self.raw.requestBufp, self.raw.mapToUrl)
    }

    pub fn get_request_url(&'a self) -> Url<'a, Self> {
        Url::new(self.raw.requestBufp, self.raw.requestUrl)
    }

    pub fn set_redirect(&mut self, redirect: bool) {
        self.raw.redirect = if redirect { 1 } else { 0 };
    }
}

pub enum TSRemapStatus {
    NoRemap,
    DidRemap,
    NoRemapStop,
    DidRemapStop,
    RemapError,
}
