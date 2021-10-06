//! Macros for interacting with PHP, mainly when the function takes variadic arguments.
//! Unforutunately, this is the best way to handle these.
//! Note that most of these will introduce unsafe into your code base.

/// Starts the PHP extension information table displayed when running `phpinfo();`
/// Must be run *before* rows are inserted into the table.
#[macro_export]
macro_rules! info_table_start {
    () => {
        unsafe { $crate::bindings::php_info_print_table_start() };
    };
}

/// Ends the PHP extension information table. Must be run *after* all rows have been inserted into the table.
#[macro_export]
macro_rules! info_table_end {
    () => {
        unsafe { $crate::bindings::php_info_print_table_end() }
    };
}

/// Sets the header for the PHP extension information table. Takes as many string arguments as required.
#[macro_export]
macro_rules! info_table_header {
    ($($element:expr),*) => {$crate::_info_table_row!(php_info_print_table_header, $($element),*)};
}

/// Adds a row to the PHP extension information table. Takes as many string arguments as required.
#[macro_export]
macro_rules! info_table_row {
    ($($element:expr),*) => {$crate::_info_table_row!(php_info_print_table_row, $($element),*)};
}

/// INTERNAL: Calls a variadic C function with the number of parameters, then following with the parameters.
#[doc(hidden)]
#[macro_export]
macro_rules! _info_table_row {
    ($fn: ident, $($element: expr),*) => {
        unsafe {
            $crate::bindings::$fn($crate::_info_table_row!(@COUNT; $($element),*) as i32, $(::std::ffi::CString::new($element).unwrap().as_ptr()),*);
        }
    };

    (@COUNT; $($element: expr),*) => {
        <[()]>::len(&[$($crate::_info_table_row![@SUBST; $element]),*])
    };
    (@SUBST; $_: expr) => { () };
}

/// Attempts to call a given PHP callable.
///
/// # Parameters
///
/// * `$fn` - The 'function' to call. Can be an [`Arg`](crate::php::args::Arg) or a
/// [`Zval`](crate::php::types::zval::Zval).
/// * ...`$param` - The parameters to pass to the function. Must be able to be converted into a
/// [`Zval`](crate::php::types::zval::Zval).
#[macro_export]
macro_rules! call_user_func {
    ($fn: expr) => {
        $fn.try_call(vec![])
    };

    ($fn: expr, $($param: expr),*) => {
        $fn.try_call(vec![$(&$param),*])
    };
}

/// Parses a given list of arguments using the [`ArgParser`](crate::php::args::ArgParser) class.
///
/// # Examples
///
/// This example parses all of the arguments. If one is invalid, execution of the function will
/// stop at the `parse_args!` macro invocation. The user is notified via PHP's argument parsing
/// system.
///
/// In this case, all of the arguments are required.
///
/// ```
/// # #[macro_use] extern crate ext_php_rs;
/// use ext_php_rs::{
///    parse_args,
///    php::{args::Arg, enums::DataType, execution_data::ExecutionData, types::zval::Zval},
/// };
///
/// pub extern "C" fn example_fn(execute_data: &mut ExecutionData, _: &mut Zval) {
///     let mut x = Arg::new("x", DataType::Long);
///     let mut y = Arg::new("y", DataType::Long);
///     let mut z = Arg::new("z", DataType::Long);
///
///     parse_args!(execute_data, x, y, z);
/// }
/// ```
///
/// This example is similar to the one above, apart from the fact that the `z` argument is not
/// required. Note the semicolon seperating the first two arguments from the second.
///
/// ```
/// use ext_php_rs::{
///    parse_args,
///    php::{args::Arg, enums::DataType, execution_data::ExecutionData, types::zval::Zval},
/// };
///
/// pub extern "C" fn example_fn(execute_data: &mut ExecutionData, _: &mut Zval) {
///     let mut x = Arg::new("x", DataType::Long);
///     let mut y = Arg::new("y", DataType::Long);
///     let mut z = Arg::new("z", DataType::Long);
///
///     parse_args!(execute_data, x, y; z);
/// }
/// ```
#[macro_export]
macro_rules! parse_args {
    ($ed: expr, $($arg: expr),*) => {{
        let parser = $ed.parser()
            $(.arg(&mut $arg))*
            .parse();
        if parser.is_err() {
            return;
        }
    }};

    ($ed: expr, $($arg: expr),* ; $($opt: expr),*) => {{
        use $crate::php::args::ArgParser;

        let parser = $ed.parser()
            $(.arg(&mut $arg))*
            .not_required()
            $(.arg(&mut $opt))*
            .parse();
        if parser.is_err() {
            return;
        }
    }};
}

/// Throws an exception and returns from the current function.
///
/// Wraps the [`throw`] function by inserting a `return` statement after throwing the exception.
///
/// [`throw`]: crate::php::exceptions::throw
///
/// # Examples
///
/// ```
/// use ext_php_rs::{throw, php::{class::ClassEntry, execution_data::ExecutionData, types::zval::Zval}};
///
/// pub extern "C" fn example_fn(execute_data: &mut ExecutionData, _: &mut Zval) {
///     let something_wrong = true;
///     if something_wrong {
///         throw!(ClassEntry::exception(), "Something is wrong!");
///     }
///
///     assert!(false); // This will not run.
/// }
/// ```
#[macro_export]
macro_rules! throw {
    ($ex: expr, $reason: expr) => {
        $crate::php::exceptions::throw($ex, $reason);
        return;
    };
}

/// Implements a set of traits required to convert types that implement [`RegisteredClass`] to and
/// from [`ZendObject`]s and [`Zval`]s. Generally, this macro should not be called directly, as it
/// is called on any type that uses the [`#[php_class]`] macro.
///
/// The following traits are implemented:
///
/// * `FromZendObject for &'a T`
/// * `FromZendObjectMut for &'a mut T`
/// * `FromZval for &'a T`
/// * `FromZvalMut for &'a mut T`
/// * `IntoZendObject for T`
/// * `IntoZval for T`
///
/// These implementations are required while we wait on the stabilisation of specialisation.
///
/// # Examples
///
/// ```
/// # use ext_php_rs::{php::types::{zval::{Zval, IntoZval, FromZval}, object::{ZendObject, RegisteredClass}}};
/// use ext_php_rs::class_derives;
///
/// struct Test {
///     a: i32,
///     b: i64
/// }
///
/// impl RegisteredClass for Test {
///     const CLASS_NAME: &'static str = "Test";
///
///     const CONSTRUCTOR: Option<ext_php_rs::php::types::object::ConstructorMeta<Self>> = None;
///
///     fn get_metadata() -> &'static ext_php_rs::php::types::object::ClassMetadata<Self> {
///         todo!()
///     }
///
///     fn get_properties<'a>(
///     ) -> std::collections::HashMap<&'static str, ext_php_rs::php::types::props::Property<'a, Self>>
///     {
///         todo!()
///     }
/// }
///
/// class_derives!(Test);
///
/// fn into_zval_test() -> Zval {
///     let x = Test { a: 5, b: 10 };
///     x.into_zval(false).unwrap()
/// }
///
/// fn from_zval_test<'a>(zv: &'a Zval) -> &'a Test {
///     <&Test>::from_zval(zv).unwrap()
/// }
/// ```
///
/// [`RegisteredClass`]: crate::php::types::object::RegisteredClass
/// [`ZendObject`]: crate::php::types::object::ZendObject
/// [`Zval`]: crate::php::types::zval::Zval
/// [`#[php_class]`]: crate::php_class
#[macro_export]
macro_rules! class_derives {
    ($type: ty) => {
        impl<'a> $crate::php::types::object::FromZendObject<'a> for &'a $type {
            #[inline]
            fn from_zend_object(
                obj: &'a $crate::php::types::object::ZendObject,
            ) -> $crate::errors::Result<Self> {
                let obj = $crate::php::types::object::ZendClassObject::<$type>::from_zend_obj(obj)
                    .ok_or($crate::errors::Error::InvalidScope)?;
                Ok(&**obj)
            }
        }

        impl<'a> $crate::php::types::object::FromZendObjectMut<'a> for &'a mut $type {
            #[inline]
            fn from_zend_object_mut(
                obj: &'a mut $crate::php::types::object::ZendObject,
            ) -> $crate::errors::Result<Self> {
                let obj =
                    $crate::php::types::object::ZendClassObject::<$type>::from_zend_obj_mut(obj)
                        .ok_or($crate::errors::Error::InvalidScope)?;
                Ok(&mut **obj)
            }
        }

        impl<'a> $crate::php::types::zval::FromZval<'a> for &'a $type {
            const TYPE: $crate::php::enums::DataType = $crate::php::enums::DataType::Object(Some(
                <$type as $crate::php::types::object::RegisteredClass>::CLASS_NAME,
            ));

            #[inline]
            fn from_zval(zval: &'a $crate::php::types::zval::Zval) -> ::std::option::Option<Self> {
                <Self as $crate::php::types::object::FromZendObject>::from_zend_object(
                    zval.object()?,
                )
                .ok()
            }
        }

        impl<'a> $crate::php::types::zval::FromZvalMut<'a> for &'a mut $type {
            const TYPE: $crate::php::enums::DataType = $crate::php::enums::DataType::Object(Some(
                <$type as $crate::php::types::object::RegisteredClass>::CLASS_NAME,
            ));

            #[inline]
            fn from_zval_mut(
                zval: &'a mut $crate::php::types::zval::Zval,
            ) -> ::std::option::Option<Self> {
                <Self as $crate::php::types::object::FromZendObjectMut>::from_zend_object_mut(
                    zval.object_mut()?,
                )
                .ok()
            }
        }

        impl $crate::php::types::object::IntoZendObject for $type {
            #[inline]
            fn into_zend_object(
                self,
            ) -> $crate::errors::Result<
                $crate::php::boxed::ZBox<$crate::php::types::object::ZendObject>,
            > {
                Ok($crate::php::types::object::ZendClassObject::new(self).into())
            }
        }

        impl $crate::php::types::zval::IntoZval for $type {
            const TYPE: $crate::php::enums::DataType = $crate::php::enums::DataType::Object(Some(
                <$type as $crate::php::types::object::RegisteredClass>::CLASS_NAME,
            ));

            #[inline]
            fn set_zval(
                self,
                zv: &mut $crate::php::types::zval::Zval,
                persistent: bool,
            ) -> $crate::errors::Result<()> {
                use $crate::php::types::object::IntoZendObject;

                self.into_zend_object()?.set_zval(zv, persistent)
            }
        }
    };
}
