/// Generates mock symbols required to generate stub files from a downstream
/// crates CLI application.
#[macro_export]
macro_rules! stub_symbols {
    () => {
        $crate::stub_symbols!(std_object_handlers, $crate::ffi::zend_object_handlers);
        $crate::stub_symbols!(zend_ce_exception, *mut $crate::ffi::zend_class_entry);
        $crate::stub_symbols!(zend_class_serialize_deny, *mut ());
        $crate::stub_symbols!(zend_class_unserialize_deny, *mut ());
        $crate::stub_symbols!(zend_string_init_interned, *mut ());

        /// Pretends to access the static stub symbols. If this function is not
        /// called, the symbols are optimised out of the resulting
        /// executable.
        pub fn link() {
            unsafe {
                ::std::convert::identity(&std_object_handlers);
                ::std::convert::identity(&zend_ce_exception);
                ::std::convert::identity(&zend_class_serialize_deny);
                ::std::convert::identity(&zend_class_unserialize_deny);
                ::std::convert::identity(&zend_string_init_interned);
            }
        }
    };
    ($s: ident, $t: ty) => {
        #[allow(non_upper_case_globals)]
        #[no_mangle]
        pub static mut $s: ::std::mem::MaybeUninit<$t> = ::std::mem::MaybeUninit::uninit();
    };
}
