#![feature(const_maybe_uninit_assume_init)]
#![feature(generic_const_exprs)]

use ext_php_rs::ffi;
use std::mem::ManuallyDrop;

macro_rules! stub_symbol {
    ($s: ident, $t: ty) => {
        #[allow(non_upper_case_globals)]
        #[used]
        #[no_mangle]
        pub static mut $s: $t = unsafe { StubSymbol::zeroed() };
    };
}

union StubSymbol<T>
where
    [(); std::mem::size_of::<T>()]: Sized,
{
    zero: [u8; std::mem::size_of::<T>()],
    obj: ManuallyDrop<T>,
}

impl<T> StubSymbol<T>
where
    [(); std::mem::size_of::<T>()]: Sized,
{
    pub const unsafe fn zeroed() -> T {
        ManuallyDrop::into_inner(
            Self {
                zero: [0; std::mem::size_of::<T>()],
            }
            .obj,
        )
    }
}

stub_symbol!(std_object_handlers, ffi::zend_object_handlers);
stub_symbol!(zend_ce_exception, *mut ffi::zend_class_entry);
stub_symbol!(zend_class_serialize_deny, *mut ());
stub_symbol!(zend_class_unserialize_deny, *mut ());
stub_symbol!(zend_string_init_interned, *mut ());

pub fn link() {
    unsafe {
        std::convert::identity(&std_object_handlers);
        std::convert::identity(&zend_ce_exception);
        std::convert::identity(&zend_class_serialize_deny);
        std::convert::identity(&zend_class_unserialize_deny);
        std::convert::identity(&zend_string_init_interned);
    }
}
