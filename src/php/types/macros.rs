pub use paste::paste;

/// Implements the [`ZendObjectOverride`] trait for the given type.
/// Also defines the static mutable object handlers for the type.
/// **MUST** be called in conjunction with the [`object_handlers_init`] macro.
///
/// # Parameters
///
/// * `$class` - The type to implement the trait for.
#[macro_export]
macro_rules! object_override_handler {
    ($class: ident) => {
        $crate::php::types::macros::paste! {
            static mut [<$class _OBJECT_HANDLERS>]: Option<*mut $crate::php::types::object::ZendObjectHandlers> = None;

            impl $crate::php::types::object::ZendObjectOverride for $class {
                extern "C" fn create_object(
                    ce: *mut $crate::php::class::ClassEntry,
                ) -> *mut $crate::php::types::object::ZendObject {
                    unsafe {
                        if [<$class _OBJECT_HANDLERS>].is_none() {
                            [<$class _OBJECT_HANDLERS>] = Some($crate::php::types::object::ZendObjectHandlers::init::<$class>());
                        }

                        $crate::php::types::object::ZendClassObject::<$class>::new_ptr(ce, [<$class _OBJECT_HANDLERS>].unwrap())
                    }
                }
            }
        }
    };
}

/// Initializes a types object handlers. This should be called at the start of
/// the module startup function which is defined by the user.
///
/// # Parameters
///
/// * `$class` - The type to initialize the handlers for.
#[macro_export]
macro_rules! object_handlers_init {
    ($class: ident) => {{
        $crate::php::types::macros::paste! {
            let ptr = $crate::php::types::object::ZendObjectHandlers::init::<$class>();
            unsafe { [<$class _OBJECT_HANDLERS>] = ptr; };
        }
    }};
}
