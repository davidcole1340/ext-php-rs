#![cfg_attr(windows, feature(abi_vectorcall))]
#![cfg(feature = "embed")]
extern crate ext_php_rs;

use ext_php_rs::builders::SapiBuilder;
use ext_php_rs::embed::{ext_php_rs_sapi_startup, Embed};
use ext_php_rs::ffi::{
    php_module_shutdown, php_module_startup, php_request_shutdown, php_request_startup,
    sapi_shutdown, sapi_startup, ZEND_RESULT_CODE_SUCCESS,
};
use ext_php_rs::zend::try_catch_first;
use ext_php_rs::{php_module, prelude::*};
use std::ffi::c_char;

static mut LAST_OUTPUT: String = String::new();

extern "C" fn output_tester(str: *const c_char, str_length: usize) -> usize {
    let char = unsafe { std::slice::from_raw_parts(str as *const u8, str_length) };
    let string = String::from_utf8_lossy(char);

    println!("{}", string);

    unsafe {
        LAST_OUTPUT = string.to_string();
    };

    str_length
}

#[test]
fn test_sapi() {
    let mut builder = SapiBuilder::new("test", "Test");
    builder = builder.ub_write_function(output_tester);

    let sapi = builder.build().unwrap().into_raw();
    let module = module::get_module();

    unsafe {
        ext_php_rs_sapi_startup();
    }

    unsafe {
        sapi_startup(sapi);
    }

    unsafe {
        php_module_startup(sapi, module);
    }

    let result = unsafe { php_request_startup() };

    assert_eq!(result, ZEND_RESULT_CODE_SUCCESS);

    let _ = try_catch_first(|| {
        let result = Embed::eval("$foo = hello_world('foo');");

        assert!(result.is_ok());

        let zval = result.unwrap();

        assert!(zval.is_string());

        let string = zval.string().unwrap();

        assert_eq!(string.to_string(), "Hello, foo!");

        let result = Embed::eval("var_dump($foo);");

        assert!(result.is_ok());
    });

    unsafe {
        php_request_shutdown(std::ptr::null_mut());
    }

    unsafe {
        php_module_shutdown();
    }

    unsafe {
        sapi_shutdown();
    }
}

#[php_module]
mod module {
    /// Gives you a nice greeting!
    ///
    /// @param string $name Your name.
    ///
    /// @return string Nice greeting!
    #[php_function]
    pub fn hello_world(name: String) -> String {
        format!("Hello, {}!", name)
    }
}
