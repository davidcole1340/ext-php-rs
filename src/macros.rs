/// Starts the PHP extension information table displayed when running `phpinfo();`
/// Must be run *before* rows are inserted into the table.
#[macro_export]
macro_rules! info_table_start {
    () => {
        unsafe { $crate::php_info_print_table_start() };
    };
}

/// Ends the PHP extension information table. Must be run *after* all rows have been inserted into the table.
#[macro_export]
macro_rules! info_table_end {
    () => {
        unsafe { $crate::php_info_print_table_end() }
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
            $crate::$fn($crate::_info_table_row!(@COUNT; $($element),*) as i32, $($crate::functions::c_str($element)),*);
        }
    };

    (@COUNT; $($element: expr),*) => {
        <[()]>::len(&[$($crate::_info_table_row![@SUBST; $element]),*])
    };
    (@SUBST; $_: expr) => { () };
}
