#[macro_export]
macro_rules! info_table_start {
    () => {
        unsafe { crate::php_info_print_table_start() };
    };
}

#[macro_export]
macro_rules! info_table_end {
    () => {
        unsafe { crate::php_info_print_table_end() }
    };
}

#[macro_export]
macro_rules! info_table_header {
    ($($element:expr),*) => {$crate::_info_table_row!(php_info_print_table_header, $($element),*)};
}

#[macro_export]
macro_rules! info_table_row {
    ($($element:expr),*) => {$crate::_info_table_row!(php_info_print_table_row, $($element),*)};
}

#[macro_export]
macro_rules! _info_table_row {
    ($fn: ident, $($element: expr),*) => {
        unsafe {
            crate::$fn($crate::_info_table_row!(@COUNT; $($element),*) as i32, $(crate::functions::c_str($element)),*);
        }
    };

    (@COUNT; $($element: expr),*) => {
        <[()]>::len(&[$($crate::_info_table_row![@SUBST; $element]),*])
    };
    (@SUBST; $_: expr) => { () };
}
