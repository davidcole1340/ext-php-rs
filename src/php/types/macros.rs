pub use paste::paste;

#[macro_export]
macro_rules! object_override_handler {
    ($class: ident) => {
        $crate::php::types::macros::paste! {
            static mut [<$class _OBJECT_HANDLERS>]: *mut $crate::php::types::object::ZendObjectHandlers = ::std::ptr::null_mut();

            impl $crate::php::types::object::ZendObjectOverride for $class {
                extern "C" fn create_object(
                    ce: *mut $crate::php::class::ClassEntry,
                ) -> *mut $crate::php::types::object::ZendObject {
                    unsafe {
                        $crate::php::types::object::ZendClassObject::<$class>::new_ptr(ce, [<$class _OBJECT_HANDLERS>])
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! object_handlers_init {
    ($class: ident) => {{
        $crate::php::types::macros::paste! {
            let ptr = $crate::php::types::object::ZendObjectHandlers::init::<$class>();
            unsafe { [<$class _OBJECT_HANDLERS>] = ptr; };
        }
    }};
}
