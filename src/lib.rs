#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub mod macros;
pub(crate) mod bindings;
pub mod functions;
pub mod php;

// Bindings used by macros. Used so that the rest of the bindings can be hidden with `pub(crate)`.
extern "C" {
    #[doc(hidden)]
    pub fn php_info_print_table_header(num_cols: ::std::os::raw::c_int, ...);
    #[doc(hidden)]
    pub fn php_info_print_table_row(num_cols: ::std::os::raw::c_int, ...);
    #[doc(hidden)]
    pub fn php_info_print_table_start();
    #[doc(hidden)]
    pub fn php_info_print_table_end();
}
