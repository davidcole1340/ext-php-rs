use std::ffi::CString;

pub fn c_str<T>(s: T) -> *const i8
where
    T: Into<Vec<u8>>
{
    let x = CString::new(s).unwrap();
    x.as_ptr()
}