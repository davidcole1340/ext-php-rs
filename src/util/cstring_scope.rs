use std::{
    ffi::{CString, NulError, c_char},
    ops::Deref,
};

// Helpful for CString which only needs to live until immediately after C call.
pub struct CStringScope(*mut c_char);

impl CStringScope {
    #[allow(dead_code)]
    pub fn new<T: Into<Vec<u8>>>(string: T) -> Result<Self, NulError> {
        Ok(Self(CString::new(string)?.into_raw()))
    }
}

impl Deref for CStringScope {
    type Target = *mut c_char;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for CStringScope {
    fn drop(&mut self) {
        // Convert back to a CString to ensure it gets dropped
        drop(unsafe { CString::from_raw(self.0) });
    }
}
