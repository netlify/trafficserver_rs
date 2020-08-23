use crate::bindings::*;
use crate::{
    headers::Headers,
    helpers::{checked_raw_str, ts_result_convert},
    transaction::GetsInternalTransactionPtr,
    url::Url,
};

use std::marker::PhantomData;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get client request from transaction")]
    GetClientRequestFailed,

    #[error("failed to get server request from transaction")]
    GetServerRequestFailed,

    #[error("failed to get url from request")]
    GetUrlFailed,
}

/// Wrapper for interacting with request structures
///
/// Used for reading the client request or cached request.
/// Or reading and manipulating the server request.
///
/// Can only be returned from certain transaction methods.
pub struct Request<'a, T: 'a> {
    bufp: TSMBuffer,
    offset: TSMLoc,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Request<'a, T> {
    pub(crate) fn new(bufp: TSMBuffer, offset: TSMLoc) -> Self {
        Self {
            bufp,
            offset,
            _phantom_data: PhantomData,
        }
    }

    pub fn get_method<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let method =
            checked_raw_str(|len| unsafe { TSHttpHdrMethodGet(self.bufp, self.offset, len) })?;
        Ok(method)
    }

    pub fn get_url<'b>(&'b self) -> Result<Url<'a, T>, crate::Error> {
        let mut loc: TSMLoc = std::ptr::null_mut();
        let ret = unsafe { TSHttpHdrUrlGet(self.bufp, self.offset, &mut loc) };
        ts_result_convert(ret).map_err(|_| Error::GetUrlFailed)?;
        Ok(Url::new(self.bufp, loc))
    }

    /// Returns the effective url for a request
    ///
    /// If the request has a Host header field
    /// (and the URL does not contain a host specifier),
    /// the host specifier the header provides is inserted
    /// into the URL. The host and scheme in the returned
    /// URL will be normalized to lower case letters
    /// (to make URL comparisons simple and fast).
    ///
    /// Uses `TSHttpHdrEffectiveUrlBufGet()` under the hood.
    #[cfg(feature = "ats-9")]
    pub fn get_effective_url<'b>(&'b self) -> Result<Url<'a, T>, crate::Error> {
        let store = vec![0; 2048];
        let length = 0;
        let ret = unsafe {
            TSHttpHdrEffectiveUrlBufGet(
                self.bufp,
                self.offset,
                store.as_mut_ptr(),
                store.len(),
                &mut length,
            )
        };
    }

    pub fn get_host<'b>(&'b self) -> Result<&str, crate::Error> {
        let host = checked_raw_str(|len| unsafe { TSHttpHdrHostGet(self.bufp, self.offset, len) })?;
        Ok(host)
    }

    pub fn get_headers<'b>(&'b self) -> Headers<'a, Self> {
        Headers::new(self.bufp, self.offset)
    }
}

pub trait GetsClientRequest
where
    Self: GetsInternalTransactionPtr + Sized,
{
    fn get_client_request(&self) -> Result<Request<Self>, crate::Error> {
        let txn = self.get_internal_transaction_pointer();
        let mut buf: TSMBuffer = std::ptr::null_mut();
        let mut loc: TSMLoc = std::ptr::null_mut();
        let ret = unsafe { TSHttpTxnClientReqGet(txn, &mut buf, &mut loc) };
        ts_result_convert(ret).map_err(|_| Error::GetClientRequestFailed)?;
        Ok(Request::new(buf, loc))
    }
}

pub trait GetsServerRequest
where
    Self: GetsInternalTransactionPtr + Sized,
{
    fn get_server_request(&self) -> Result<Request<Self>, crate::Error> {
        let txn = self.get_internal_transaction_pointer();
        let mut buf: TSMBuffer = std::ptr::null_mut();
        let mut loc: TSMLoc = std::ptr::null_mut();
        let ret = unsafe { TSHttpTxnServerReqGet(txn, &mut buf, &mut loc) };
        ts_result_convert(ret).map_err(|_| Error::GetServerRequestFailed)?;
        Ok(Request::new(buf, loc))
    }
}
