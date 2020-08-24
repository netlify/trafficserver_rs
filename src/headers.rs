//! This module provides access to the HTTP MIME headers
//! of a request or response.
//!
//! Not to be confused with the request/response metadata
//! which is also called "headers" in TrafficServer

use crate::bindings::*;
use crate::helpers::checked_raw_str;
use std::marker::PhantomData;

/// An iterator over header field values
///
/// Skips any invalid strings
pub struct HeaderValueIter<'a, 'b, T: 'a> {
    field: &'b HeaderField<'a, 'b, T>,
    index: usize,
    length: usize,
}

impl<'a, 'b, T> Iterator for HeaderValueIter<'a, 'b, T> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.length {
            let res = checked_raw_str(|len| unsafe {
                TSMimeHdrFieldValueStringGet(
                    self.field.headers.bufp,
                    self.field.headers.offset,
                    self.field.raw_field,
                    self.index as i32,
                    len,
                )
            });
            self.index += 1;
            if let Ok(res) = res {
                return Some(res);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.length))
    }
}

/// A single MIME header field
///
/// Has a name and contains multiple values.
pub struct HeaderField<'a, 'b, T: 'a> {
    headers: &'b Headers<'a, T>,
    raw_field: TSMLoc,
}

impl<'a, 'b, T> Drop for HeaderField<'a, 'b, T> {
    fn drop(&mut self) {
        unsafe { TSHandleMLocRelease(self.headers.bufp, self.headers.offset, self.raw_field) };
    }
}

impl<'a, 'b, T> HeaderField<'a, 'b, T> {
    /// Returns the name of this field
    pub fn get_name(&self) -> Result<&str, crate::Error> {
        let name = checked_raw_str(|len| unsafe {
            TSMimeHdrFieldNameGet(self.headers.bufp, self.headers.offset, self.raw_field, len)
        })?;
        Ok(name)
    }

    /// Returns an Iterator over all values for this field
    ///
    /// Any invalid strings will be skipped.
    pub fn get_value_iter(&'b self) -> HeaderValueIter<'a, 'b, T> {
        let length = unsafe {
            TSMimeHdrFieldValuesCount(self.headers.bufp, self.headers.offset, self.raw_field)
        } as usize;
        HeaderValueIter {
            field: self,
            index: 0,
            length,
        }
    }

    /// Returns the first value for this field
    pub fn get_value(&self) -> Result<&str, crate::Error> {
        let val = checked_raw_str(|len| unsafe {
            TSMimeHdrFieldValueStringGet(
                self.headers.bufp,
                self.headers.offset,
                self.raw_field,
                0,
                len,
            )
        })?;
        Ok(val)
    }
}

/// Iterator over all MIME header fields
/// of a HTTP request or response
pub struct HeaderIter<'a, 'b, T: 'a> {
    headers: &'b Headers<'a, T>,
    index: usize,
    length: usize,
}

impl<'a, 'b, T> Iterator for HeaderIter<'a, 'b, T> {
    type Item = HeaderField<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }

        let offset =
            unsafe { TSMimeHdrFieldGet(self.headers.bufp, self.headers.offset, self.index as i32) };
        self.index += 1;
        if offset.is_null() {
            return None;
        }

        Some(HeaderField {
            headers: self.headers,
            raw_field: offset,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.length))
    }
}

/// Wrapper for interacting with MIME header fields
/// of a HTTP request or response
pub struct Headers<'a, T: 'a> {
    bufp: TSMBuffer,
    offset: TSMLoc,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Headers<'a, T> {
    pub(crate) fn new(bufp: TSMBuffer, offset: TSMLoc) -> Self {
        Self {
            bufp,
            offset,
            _phantom_data: PhantomData,
        }
    }

    pub fn iter<'b>(&'b self) -> HeaderIter<'a, 'b, T> {
        let length = unsafe { TSMimeHdrFieldsCount(self.bufp, self.offset) } as usize;
        HeaderIter {
            headers: self,
            index: 0,
            length,
        }
    }

    pub fn find<'b>(&'b self, name: &str) -> Option<HeaderField<'a, 'b, T>> {
        let char_ptr = name.as_bytes().as_ptr() as *const i8;
        let raw_field =
            unsafe { TSMimeHdrFieldFind(self.bufp, self.offset, char_ptr, name.len() as i32) };
        if raw_field.is_null() {
            return None;
        }
        Some(HeaderField {
            headers: self,
            raw_field,
        })
    }
}

impl<'a, 'b, T> IntoIterator for &'b Headers<'a, T> {
    type Item = HeaderField<'a, 'b, T>;
    type IntoIter = HeaderIter<'a, 'b, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
