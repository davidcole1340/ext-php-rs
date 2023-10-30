#![cfg_attr(windows, feature(abi_vectorcall))]
extern crate ext_php_rs;

#[cfg(feature = "embed")]
use ext_php_rs::builders::SapiBuilder;
#[cfg(feature = "embed")]
use ext_php_rs::embed::{ext_php_rs_sapi_startup, Embed};
use ext_php_rs::ffi::{
    php_module_shutdown, php_module_startup, php_request_shutdown, php_request_startup,
    sapi_shutdown, sapi_startup, ZEND_RESULT_CODE_SUCCESS,
};
use ext_php_rs::prelude::*;
use ext_php_rs::zend::try_catch;

#[test]
#[cfg(feature = "embed")]
fn test_sapi() {
    let builder = SapiBuilder::new("test", "Test");
    let sapi = builder.build().unwrap().into_raw();
    let module = get_module();

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

    let _ = try_catch(
        || {
            let result = Embed::eval("$foo = hello_world('foo');");

            assert!(result.is_ok());

            let zval = result.unwrap();

            assert!(zval.is_string());

            let string = zval.string().unwrap();

            assert_eq!(string.to_string(), "Hello, foo!");
        },
        true,
    );

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

/// Gives you a nice greeting!
///
/// @param string $name Your name.
///
/// @return string Nice greeting!
#[php_function]
pub fn hello_world(name: String) -> String {
    format!("Hello, {}!", name)
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
