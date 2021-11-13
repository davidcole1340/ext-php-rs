use crate::ffi;
use std::mem::MaybeUninit;

macro_rules! stub_symbol {
    ($s: ident, $t: ty) => {
        #[allow(non_upper_case_globals)]
        #[used]
        #[no_mangle]
        pub static mut $s: MaybeUninit<$t> = MaybeUninit::uninit();
    };
}

stub_symbol!(std_object_handlers, ffi::zend_object_handlers);
stub_symbol!(zend_ce_exception, *mut ffi::zend_class_entry);
stub_symbol!(zend_class_serialize_deny, *mut ());
stub_symbol!(zend_class_unserialize_deny, *mut ());
stub_symbol!(zend_string_init_interned, *mut ());

/// Pretends to access the static stub symbols. If this function is not called,
/// the symbols are optimised out of the resulting executable.
pub fn link() {
    unsafe {
        std::convert::identity(&std_object_handlers);
        std::convert::identity(&zend_ce_exception);
        std::convert::identity(&zend_class_serialize_deny);
        std::convert::identity(&zend_class_unserialize_deny);
        std::convert::identity(&zend_string_init_interned);
    }
}
