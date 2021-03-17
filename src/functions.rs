//! Helper functions useful for interacting with PHP and Zend.

use std::ffi::CString;

/// Takes a Rust string object, converts it into a C string
/// and then releases the string to the C world.
///
/// Note that strings produced by this function *will not* be freed by
/// Rust, and this can cause memory leaks.
///
/// # Examples
///
/// ```
/// use std::ffi::CString;
/// use php_rs::functions::c_str;
///
/// let mut ptr = c_str("Hello");
///
/// unsafe {
///     assert_eq!(b'H', *ptr as u8);
///     assert_eq!(b'e', *ptr.offset(1) as u8);
///     assert_eq!(b'l', *ptr.offset(2) as u8);
///     assert_eq!(b'l', *ptr.offset(3) as u8);
///     assert_eq!(b'o', *ptr.offset(4) as u8);
///     assert_eq!(b'\0', *ptr.offset(5) as u8);
///
///     // reclaim string and release memory
///     let _ = CString::from_raw(ptr);
/// }
/// ```
pub fn c_str<S>(s: S) -> *const i8
where
    S: AsRef<str>,
{
    CString::into_raw(CString::new(s.as_ref()).unwrap())
}
