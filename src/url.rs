use crate::{
    bindings::*,
    helpers::{checked_raw_str, checked_raw_string},
};
use std::marker::PhantomData;

pub struct Url<'a, T: 'a> {
    bufp: TSMBuffer,
    offset: TSMLoc,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Url<'a, T> {
    pub(crate) fn new(bufp: TSMBuffer, offset: TSMLoc) -> Self {
        Self {
            bufp,
            offset,
            _phantom_data: PhantomData,
        }
    }

    pub fn get_host<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlHostGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_scheme<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlSchemeGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_user<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlUserGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_password<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlPasswordGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_port(&self) -> i32 {
        unsafe { TSUrlPortGet(self.bufp, self.offset) }
    }

    pub fn get_path<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlPathGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_query<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res = checked_raw_str(|len| unsafe { TSUrlHttpQueryGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_params<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res =
            checked_raw_str(|len| unsafe { TSUrlHttpParamsGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }

    pub fn get_fragment<'b>(&'b self) -> Result<&'a str, crate::Error> {
        let res =
            checked_raw_str(|len| unsafe { TSUrlHttpFragmentGet(self.bufp, self.offset, len) })?;
        Ok(res)
    }
}

impl<T> std::fmt::Display for Url<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = checked_raw_string(|len| unsafe { TSUrlStringGet(self.bufp, self.offset, len) })
            .map_err(|_| std::fmt::Error)?;
        f.write_str(&val)
    }
}
