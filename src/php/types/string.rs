use crate::{
    bindings::{
        zend_string, zend_strpprintf, GC_FLAGS_MASK, GC_FLAGS_SHIFT, GC_INFO_SHIFT, IS_STR_INTERNED,
    },
    functions::c_str,
};

/// String type used in the Zend internals.
/// The actual size of the 'string' differs, as the
/// end of this struct is only 1 char long, but the length
/// inside the struct defines how many characters are in the string.
pub type ZendString = zend_string;

impl ZendString {
    /// Creates a new Zend string.
    ///
    /// Note that this returns a raw pointer, and will not be freed by
    /// Rust.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    pub fn new<S>(str_: S) -> *mut Self
    where
        S: AsRef<str>,
    {
        let str_ = str_.as_ref();
        unsafe { zend_strpprintf(str_.len() as u64, c_str(str_)) }
    }

    /// Translation of the `ZSTR_IS_INTERNED` macro.
    /// zend_string.h:76
    pub(crate) unsafe fn is_interned(&self) -> bool {
        (((self.gc.u.type_info >> GC_INFO_SHIFT) & (GC_FLAGS_MASK >> GC_FLAGS_SHIFT))
            & IS_STR_INTERNED)
            != 0
    }
}
