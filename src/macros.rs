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
