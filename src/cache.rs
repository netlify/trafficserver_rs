use crate::bindings::*;
use crate::continuations::continuation_callback;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open cache key")]
    OpenFailed(isize),

    #[error("unexpected event")]
    UnexpectedEvent,

    #[error("channel closed")]
    ChannelClosed,
}

pub struct CacheObject {
    vconn: TSVConn,
    buffer: Option<TSIOBuffer>,
}

impl CacheObject {
    fn new(vconn: TSVConn) -> Self {
        Self {
            vconn,
            buffer: None,
        }
    }

    // todo: cache read methods
}

impl Drop for CacheObject {
    fn drop(&mut self) {
        self.buffer
            .take()
            .map(|bufp| unsafe { TSIOBufferDestroy(bufp) });
        unsafe { TSVConnClose(self.vconn) };
    }
}

pub async fn cache_read(cache_key: &str) -> Result<CacheObject, Error> {
    let key = unsafe { TSCacheKeyCreate() };
    unsafe { TSCacheKeyDigestSet(key, cache_key.as_ptr() as *const i8, cache_key.len() as i32) };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let mut sender = Some(tx);
    let contp = continuation_callback(move |event, edata| {
        let tx = sender.take().ok_or(())?;
        match event {
            TSEvent_TS_EVENT_CACHE_OPEN_READ => {
                let vconn = edata as TSVConn;
                tx.send(Ok(CacheObject::new(vconn)))
            }
            TSEvent_TS_EVENT_CACHE_OPEN_READ_FAILED => {
                let err_code = edata as isize;
                tx.send(Err(Error::OpenFailed(err_code)))
            }
            _ => tx.send(Err(Error::UnexpectedEvent)),
        }
        .map_err(|_| ())
    });

    unsafe { TSCacheRead(contp, key) };

    let res = rx.await.map_err(|_| Error::ChannelClosed)?;

    unsafe { TSCacheKeyDestroy(key) };
    res
}
