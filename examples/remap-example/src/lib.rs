use trafficserver_rs::*;

use std::os::raw::{c_char, c_int, c_void};
use transaction::{hook_scopes::RemapHook, Transaction};

#[no_mangle]
pub extern "C" fn TSRemapInit(
    _api_info: *mut TSRemapInterface,
    _errbuf: *mut c_char,
    _errbuf_size: c_int,
) -> TSReturnCode {
    eprintln!("remap plugin: remap init");
    TSReturnCode_TS_SUCCESS
}

#[no_mangle]
pub extern "C" fn TSRemapDoRemap(
    _ih: *mut c_void,
    txn: TSHttpTxn,
    rri: TSRemapRequestInfo,
) -> TSRemapStatus {
    eprintln!("remap plugin: doing remap");

    let remap_info = unsafe { RemapRequestInfo::new(rri) };

    match remap_info.get_request_url().get_host() {
        Ok(host) => eprintln!("host: {}", host),
        Err(e) => eprintln!("error getting host: {:?}", e),
    }

    let transaction = unsafe { Transaction::<'_, RemapHook>::new(txn) };
    use request::GetsClientRequest;
    match transaction.get_client_request().and_then(|r| r.get_url()) {
        Ok(url) => {
            match url.get_path() {
                Ok(path) => eprintln!("path: {}", path),
                Err(e) => eprintln!("failed to get path: {}", e),
            };
            eprintln!("port: {}", url.get_port());
            eprintln!("full url: {}", url);
        }
        Err(e) => eprintln!("failed to get url: {}", e),
    }

    TSRemapStatus::NoRemap
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
    eprintln!("remap plugin: remap new instance");
    TSReturnCode_TS_SUCCESS
}

#[no_mangle]
pub extern "C" fn TSRemapDeleteInstance(_arg1: *mut c_void) {}

#[no_mangle]
pub extern "C" fn TSRemapOSResponse(_ih: *mut c_void, _rh: TSHttpTxn, _os_response_type: c_int) {}
