use crate::{
    bindings::*,
    headers::Headers,
    helpers::{checked_raw_str, ts_result_convert},
    status::Status,
    transaction::GetsInternalTransactionPtr,
};

use std::marker::PhantomData;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get client response from transaction")]
    GetClientResponseFailed,

    #[error("failed to get server response from transaction")]
    GetServerResponseFailed,
}

/// Wrapper for interacting with responses
///
/// Used for reading the client, server or cached responses.
///
/// Can only be returned from certain transaction methods.
pub struct Response<'a, T: 'a> {
    bufp: TSMBuffer,
    offset: TSMLoc,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Response<'a, T> {
    pub(crate) fn new(bufp: TSMBuffer, offset: TSMLoc) -> Self {
        Self {
            bufp,
            offset,
            _phantom_data: PhantomData,
        }
    }

    pub fn get_status(&self) -> Option<Status> {
        let raw_status = unsafe { TSHttpHdrStatusGet(self.bufp, self.offset) };
        if raw_status == TSHttpStatus_TS_HTTP_STATUS_NONE {
            return None;
        }
        use std::convert::TryFrom;
        Status::try_from(raw_status).ok()
    }

    pub fn get_reason<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res =
            checked_raw_str(|len| unsafe { TSHttpHdrReasonGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_headers<'b>(&'b self) -> Headers<'a, Self> {
        Headers::new(self.bufp, self.offset)
    }
}

pub trait GetsServerResponse
where
    Self: GetsInternalTransactionPtr + Sized,
{
    fn get_server_response(&self) -> Result<Response<Self>, crate::Error> {
        let txn = self.get_internal_transaction_pointer();
        let mut buf: TSMBuffer = std::ptr::null_mut();
        let mut loc: TSMLoc = std::ptr::null_mut();
        let ret = unsafe { TSHttpTxnServerRespGet(txn, &mut buf, &mut loc) };
        ts_result_convert(ret).map_err(|_| Error::GetServerResponseFailed)?;
        Ok(Response::new(buf, loc))
    }
}

pub trait GetsClientResponse
where
    Self: GetsInternalTransactionPtr + Sized,
{
    fn get_client_response(&self) -> Result<Response<Self>, crate::Error> {
        let txn = self.get_internal_transaction_pointer();
        let mut buf: TSMBuffer = std::ptr::null_mut();
        let mut loc: TSMLoc = std::ptr::null_mut();
        let ret = unsafe { TSHttpTxnClientRespGet(txn, &mut buf, &mut loc) };
        ts_result_convert(ret).map_err(|_| Error::GetServerResponseFailed)?;
        Ok(Response::new(buf, loc))
    }
}
