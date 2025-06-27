//! Raw FFI bindings to the Zend API.

#![allow(clippy::all)]
#![allow(warnings)]

use std::ffi::{c_char, c_int, c_void};

#[link(name = "wrapper")]
extern "C" {
    pub fn ext_php_rs_embed_callback(
        argc: c_int,
        argv: *mut *mut c_char,
        func: unsafe extern "C" fn(*const c_void) -> *const c_void,
        ctx: *const c_void,
    ) -> *mut c_void;

    pub fn ext_php_rs_sapi_startup();
    pub fn ext_php_rs_php_error(
        type_: ::std::os::raw::c_int,
        error_msg: *const ::std::os::raw::c_char,
        ...
    );
}
