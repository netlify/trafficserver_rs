extern crate trafficserver_rs;
use trafficserver_rs::*;

use std::os::raw::{c_char, c_int, c_void};

#[no_mangle]
pub extern "C" fn TSRemapInit(
        _api_info: *mut TSRemapInterface,
        _errbuf: *mut c_char,
        _errbuf_size: c_int,
    ) -> TSReturnCode {

        ts_debug("remap-example", &format!("remap init: {}", ts_config_dir_get()));

        TSReturnCode_TS_SUCCESS
}

#[no_mangle]
pub extern "C" fn TSRemapDoRemap(
        _ih: *mut c_void,
        txn: TSHttpTxn,
        rri: *mut TSRemapRequestInfo,
    ) -> TSRemapStatus {
        ts_debug("remap-example", "remap do remap");

        let url = match remap_request_url(txn, rri) {
            Err(err) => {
                ts_error(err.to_string().as_ref());
                return TSRemapStatus::RemapError;
            },
            Ok(u) => u
        };

        let headers = match remap_request_headers(rri) {
            Err(err) => {
                ts_error(&err);
                return TSRemapStatus::RemapError;
            },
            Ok(h) => h
        };

        ts_debug("remap-example", &format!("request url: {}", url));
        ts_debug("remap-example", &format!("request headers size: {}", headers.len()));


        TSRemapStatus::DidRemap
}

#[no_mangle]
pub extern "C" fn TSRemapDone() {}

#[no_mangle]
pub extern "C" fn TSRemapNewInstance(
        _argc: c_int,
        _argv: *mut *mut c_char,
        _ih: *mut *mut c_void,
        _errbuf: *mut c_char,
        _errbuf_size: c_int,
    ) -> TSReturnCode {
        ts_debug("remap-example", "remap new instance");
        TSReturnCode_TS_SUCCESS
}

#[no_mangle]
pub extern "C" fn TSRemapDeleteInstance(_arg1: *mut c_void) {}

#[no_mangle]
pub extern "C" fn TSRemapOSResponse(
        _ih: *mut c_void,
        _rh: TSHttpTxn,
        _os_response_type: c_int,
    ) {
}