use crate::bindings::*;
use ::std::os::raw::c_int;
use ::std::os::raw::c_void;

pub extern "C" fn continuation_raw_handler_once<C>(
    contp: TSCont,
    event: TSEvent,
    edata: *mut c_void,
) -> c_int
where
    C: FnOnce(TSEvent, *mut c_void) -> Result<(), ()> + 'static,
{
    let cont_data = unsafe { TSContDataGet(contp) as *mut C };
    let cb = unsafe { Box::from_raw(cont_data) };
    let res = cb(event, edata);
    unsafe { TSContDestroy(contp) };
    match res {
        Ok(_) => TSReturnCode_TS_SUCCESS,
        Err(_) => TSReturnCode_TS_ERROR,
    }
}

pub extern "C" fn continuation_raw_handler<C>(
    contp: TSCont,
    event: TSEvent,
    edata: *mut c_void,
) -> c_int
where
    C: FnMut(TSEvent, *mut c_void) -> Result<(), ()> + 'static,
{
    let cont_data = unsafe { TSContDataGet(contp) as *mut C };
    let cb = unsafe { &mut *cont_data };
    let res = cb(event, edata);
    match res {
        Ok(_) => TSReturnCode_TS_SUCCESS,
        Err(_) => TSReturnCode_TS_ERROR,
    }
}

type RawHandler = unsafe extern "C" fn(
    contp: TSCont,
    event: TSEvent,
    edata: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int;

fn continuation_wrapper<C>(cb: C, raw_handler: RawHandler) -> TSCont {
    let contp = unsafe { TSContCreate(Some(raw_handler), TSMutexCreate()) };
    let boxed_cb = Box::new(cb);
    unsafe { TSContDataSet(contp, Box::into_raw(boxed_cb) as *mut c_void) };
    contp
}

/// Create a continuation with a callback to be called once.
///
/// Dropped after first use.
pub fn continuation_callback<C>(cb: C) -> TSCont
where
    C: FnOnce(TSEvent, *mut c_void) -> Result<(), ()> + 'static,
{
    continuation_wrapper(cb, continuation_raw_handler_once::<C>)
}

/// Create a continuation with a callback to be called often.
///
/// Make sure to call `continuation_handler_free` when deregistering.
pub fn continuation_handler<C>(cb: C) -> TSCont
where
    C: FnMut(TSEvent, *mut c_void) -> Result<(), ()> + 'static,
{
    continuation_wrapper(cb, continuation_raw_handler::<C>)
}

pub unsafe fn continuation_handler_free(contp: TSCont) {
    let data = TSContDataGet(contp);
    Box::from_raw(data);
    TSContDestroy(contp);
}
