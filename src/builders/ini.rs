use std::ops::Deref;
use std::ffi::{c_char, CStr, CString, NulError};
use crate::ffi::{
    php_ini_builder,
    php_ini_builder_prepend,
    php_ini_builder_unquoted,
    php_ini_builder_quoted,
    php_ini_builder_define
};

// Helpful for CString which only needs to live until immediately after C call.
struct CStringScope(*mut c_char);

impl CStringScope {
    fn new<T: Into<Vec<u8>>>(string: T) -> Result<Self, NulError> {
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
        drop(unsafe { CString::from_raw(self.0) })
    }
}

/// A builder for creating INI configurations.
pub type IniBuilder = php_ini_builder;

impl IniBuilder {
    /// Creates a new INI builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// ```
    pub fn new() -> IniBuilder {
         IniBuilder {
            value: std::ptr::null_mut(),
            length: 0,
        }
    }

    /// Appends a value to the INI builder.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// builder.prepend("foo=bar");
    /// ```
    pub fn prepend<V: AsRef<str>>(&mut self, value: V) -> Result<(), NulError> {
        let value = value.as_ref();
        let raw = CStringScope::new(value)?;

        unsafe {
            php_ini_builder_prepend(self, *raw, value.len());
        }

        Ok(())
    }

    /// Appends an unquoted name-value pair to the INI builder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pair.
    /// * `value` - The value of the pair.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// builder.unquoted("foo", "bar");
    /// ```
    pub fn unquoted<N, V>(&mut self, name: N, value: V) -> Result<(), NulError>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        let name = name.as_ref();
        let value = value.as_ref();

        let raw_name = CStringScope::new(name)?;
        let raw_value = CStringScope::new(value)?;

        unsafe {
            php_ini_builder_unquoted(self, *raw_name, name.len(), *raw_value, value.len());
        }

        Ok(())
    }

    /// Appends a quoted name-value pair to the INI builder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pair.
    /// * `value` - The value of the pair.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// builder.quoted("foo", "bar");
    /// ```
    pub fn quoted<N, V>(&mut self, name: N, value: V) -> Result<(), NulError>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        let name = name.as_ref();
        let value = value.as_ref();

        let raw_name = CStringScope::new(name)?;
        let raw_value = CStringScope::new(value)?;

        unsafe {
            php_ini_builder_quoted(self, *raw_name, name.len(), *raw_value, value.len());
        }

        Ok(())
    }

    /// Defines a value in the INI builder.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to define.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// builder.define("foo=bar");
    /// ```
    pub fn define<V: AsRef<str>>(&mut self, value: V) -> Result<(), NulError> {
        let value = value.as_ref();
        let raw = CStringScope::new(value)?;

        unsafe {
            php_ini_builder_define(self, *raw);
        }

        Ok(())
    }

    /// Finishes building the INI configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ext_php_rs::builders::IniBuilder;
    /// let mut builder = IniBuilder::new();
    /// let ini = builder.finish();
    /// ```
    pub fn finish(&mut self) -> *mut i8 {
        if self.value.is_null() {
            return std::ptr::null_mut();
        }

        unsafe { CStr::from_ptr(self.value) }.as_ptr() as *mut c_char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ini_builder_prepend() {
        let mut builder = IniBuilder::new();
        builder.prepend("foo=bar").unwrap();

        let ini = builder.finish();
        assert!(!ini.is_null());
        assert_eq!(unsafe { CStr::from_ptr(ini) }.to_str().unwrap(), "foo=bar");
    }

    #[test]
    fn test_ini_builder_unquoted() {
        let mut builder = IniBuilder::new();
        builder.unquoted("baz", "qux").unwrap();

        let ini = builder.finish();
        assert!(!ini.is_null());
        assert_eq!(unsafe { CStr::from_ptr(ini) }.to_str().unwrap(), "baz=qux\n");
    }

    #[test]
    fn test_ini_builder_quoted() {
        let mut builder = IniBuilder::new();
        builder.quoted("quux", "corge").unwrap();

        let ini = builder.finish();
        assert!(!ini.is_null());
        assert_eq!(unsafe { CStr::from_ptr(ini) }.to_str().unwrap(), "quux=\"corge\"\n");
    }

    #[test]
    fn test_ini_builder_define() {
        let mut builder = IniBuilder::new();
        builder.define("grault=garply").unwrap();

        let ini = builder.finish();
        assert!(!ini.is_null());
        assert_eq!(unsafe { CStr::from_ptr(ini) }.to_str().unwrap(), "grault=garply\n");
    }
}
