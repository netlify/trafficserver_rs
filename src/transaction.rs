use crate::{
    bindings::*,
    helpers::ts_result_convert,
    request::{GetsClientRequest, GetsServerRequest},
};
use std::marker::PhantomData;

static TS_CRUUID_STRING_LEN: usize = 58;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get client request uuid from transaction")]
    GetClientRequestUuidFailed,

    #[error("invalid string: {0}")]
    InvalidString(std::string::FromUtf8Error),
}

/// Wrapper for interacting with a HTTP transaction.
///
/// The generic parameter is used to restrict access
/// to requests and responses depending on what hook
/// was called.
///
/// Pick a struct from `hook_scopes` to select which
/// hook the transaction is valid for.
pub struct Transaction<'a, T: 'a> {
    txnp: TSHttpTxn,
    _transaction_stage: PhantomData<&'a T>,
}

pub trait GetsInternalTransactionPtr {
    fn get_internal_transaction_pointer(&self) -> TSHttpTxn;
}

impl<'a, T> GetsInternalTransactionPtr for Transaction<'a, T> {
    fn get_internal_transaction_pointer(&self) -> TSHttpTxn {
        self.txnp
    }
}

pub mod hook_scopes {
    pub trait AllowsGetClientRequest {}
    pub trait AllowsGetServerRequest {}

    pub struct RemapHook;
    impl AllowsGetClientRequest for RemapHook {}

    pub struct ReadRequestHeaderHook;
    impl AllowsGetClientRequest for ReadRequestHeaderHook {}

    pub struct SendRequestHeaderHook;
    impl AllowsGetClientRequest for SendRequestHeaderHook {}
    impl AllowsGetServerRequest for SendRequestHeaderHook {}
}

impl<'a, T> GetsClientRequest for Transaction<'a, T> where T: hook_scopes::AllowsGetClientRequest {}

impl<'a, T> GetsServerRequest for Transaction<'a, T> where T: hook_scopes::AllowsGetServerRequest {}

impl<'a, T> Transaction<'a, T> {
    pub unsafe fn new(txnp: TSHttpTxn) -> Self {
        Self {
            txnp,
            _transaction_stage: PhantomData,
        }
    }

    /// Continues executing the transaction
    ///
    /// Consumes the transaction because it might not
    /// be safe to use after calling this.
    pub fn resume(self) {
        unsafe { TSHttpTxnReenable(self.txnp, TSEvent_TS_EVENT_HTTP_CONTINUE) };
    }

    /// Aborts the transaction
    ///
    /// Consumes the transaction because it might not
    /// be safe to use after calling this.
    pub fn abort(self) {
        unsafe { TSHttpTxnReenable(self.txnp, TSEvent_TS_EVENT_HTTP_ERROR) };
    }

    /// Get the client request uuid assigned to the transaction
    pub fn client_request_uuid(&self) -> Result<String, crate::Error> {
        let mut uuid_slice: Vec<u8> = vec![0; TS_CRUUID_STRING_LEN];
        let ret = unsafe { TSClientRequestUuidGet(self.txnp, uuid_slice.as_mut_ptr() as *mut i8) };
        ts_result_convert(ret).map_err(|_| Error::GetClientRequestUuidFailed)?;

        // trim any trailing zero-bytes
        let uuid_slice = uuid_slice.into_iter().take_while(|b| *b != 0).collect();

        let res = String::from_utf8(uuid_slice).map_err(|e| Error::InvalidString(e))?;
        Ok(res)
    }
}
