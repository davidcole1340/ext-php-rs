//! Provides implementations for running php code from rust.
//! It only works on linux for now and you should have `php-embed` installed
//!
//! This crate was only test with PHP 8.2 please report any issue with other
//! version You should only use this crate for test purpose, it's not production
//! ready

mod ffi;
mod sapi;

use crate::boxed::ZBox;
use crate::embed::ffi::ext_php_rs_embed_callback;
use crate::ffi::{
    _zend_file_handle__bindgen_ty_1, php_execute_script, zend_eval_string, zend_file_handle,
    zend_stream_init_filename, ZEND_RESULT_CODE_SUCCESS,
};
use crate::types::{ZendObject, Zval};
use crate::zend::{panic_wrapper, try_catch, ExecutorGlobals};
use parking_lot::{const_rwlock, RwLock};
use std::ffi::{c_char, c_void, CString, NulError};
use std::panic::{resume_unwind, RefUnwindSafe};
use std::path::Path;
use std::ptr::null_mut;

pub use ffi::ext_php_rs_sapi_startup;
pub use sapi::SapiModule;

pub struct Embed;

#[derive(Debug)]
pub enum EmbedError {
    InitError,
    ExecuteError(Option<ZBox<ZendObject>>),
    ExecuteScriptError,
    InvalidEvalString(NulError),
    InvalidPath,
    CatchError,
}

impl EmbedError {
    pub fn is_bailout(&self) -> bool {
        matches!(self, EmbedError::CatchError)
    }
}

static RUN_FN_LOCK: RwLock<()> = const_rwlock(());

impl Embed {
    /// Run a php script from a file
    ///
    /// This function will only work correctly when used inside the `Embed::run`
    /// function otherwise behavior is unexpected
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The script was executed successfully
    /// * `Err(EmbedError)` - An error occured during the execution of the
    ///   script
    ///
    /// # Example
    ///
    /// ```
    /// use ext_php_rs::embed::Embed;
    ///
    /// Embed::run(|| {
    ///     let result = Embed::run_script("src/embed/test-script.php");
    ///
    ///     assert!(result.is_ok());
    /// });
    /// ```
    pub fn run_script<P: AsRef<Path>>(path: P) -> Result<(), EmbedError> {
        let path = match path.as_ref().to_str() {
            Some(path) => match CString::new(path) {
                Ok(path) => path,
                Err(err) => return Err(EmbedError::InvalidEvalString(err)),
            },
            None => return Err(EmbedError::InvalidPath),
        };

        let mut file_handle = zend_file_handle {
            handle: _zend_file_handle__bindgen_ty_1 { fp: null_mut() },
            filename: null_mut(),
            opened_path: null_mut(),
            type_: 0,
            primary_script: false,
            in_list: false,
            buf: null_mut(),
            len: 0,
        };

        unsafe {
            zend_stream_init_filename(&mut file_handle, path.as_ptr());
        }

        let exec_result = try_catch(|| unsafe { php_execute_script(&mut file_handle) });

        match exec_result {
            Err(_) => Err(EmbedError::CatchError),
            Ok(true) => Ok(()),
            Ok(false) => Err(EmbedError::ExecuteScriptError),
        }
    }

    /// Start and run embed sapi engine
    ///
    /// This function will allow to run php code from rust, the same PHP context
    /// is keep between calls inside the function passed to this method.
    /// Which means subsequent calls to `Embed::eval` or `Embed::run_script`
    /// will be able to access variables defined in previous calls
    ///
    /// # Returns
    ///
    /// * R - The result of the function passed to this method
    ///
    /// R must implement [`Default`] so it can be returned in case of a bailout
    ///
    /// # Example
    ///
    /// ```
    /// use ext_php_rs::embed::Embed;
    ///
    /// Embed::run(|| {
    ///    let _ = Embed::eval("$foo = 'foo';");
    ///    let foo = Embed::eval("$foo;");
    ///    assert!(foo.is_ok());
    ///    assert_eq!(foo.unwrap().string().unwrap(), "foo");
    /// });
    /// ```
    pub fn run<R, F: FnMut() -> R + RefUnwindSafe>(func: F) -> R
    where
        R: Default,
    {
        // @TODO handle php thread safe
        //
        // This is to prevent multiple threads from running php at the same time
        // At some point we should detect if php is compiled with thread safety and
        // avoid doing that in this case
        let _guard = RUN_FN_LOCK.write();

        let panic = unsafe {
            ext_php_rs_embed_callback(
                0,
                null_mut(),
                panic_wrapper::<R, F>,
                &func as *const F as *const c_void,
            )
        };

        // This can happen if there is a bailout
        if panic.is_null() {
            return R::default();
        }

        match unsafe { *Box::from_raw(panic as *mut std::thread::Result<R>) } {
            Ok(r) => r,
            Err(err) => {
                // we resume the panic here so it can be catched correctly by the test framework
                resume_unwind(err);
            }
        }
    }

    /// Evaluate a php code
    ///
    /// This function will only work correctly when used inside the `Embed::run`
    /// function
    ///
    /// # Returns
    ///
    /// * `Ok(Zval)` - The result of the evaluation
    /// * `Err(EmbedError)` - An error occured during the evaluation
    ///
    /// # Example
    ///
    /// ```
    /// use ext_php_rs::embed::Embed;
    ///
    /// Embed::run(|| {
    ///    let foo = Embed::eval("$foo = 'foo';");
    ///    assert!(foo.is_ok());
    /// });
    /// ```
    pub fn eval(code: &str) -> Result<Zval, EmbedError> {
        let cstr = match CString::new(code) {
            Ok(cstr) => cstr,
            Err(err) => return Err(EmbedError::InvalidEvalString(err)),
        };

        let mut result = Zval::new();

        let exec_result = try_catch(|| unsafe {
            zend_eval_string(
                cstr.as_ptr() as *const c_char,
                &mut result,
                b"run\0".as_ptr() as *const _,
            )
        });

        match exec_result {
            Err(_) => Err(EmbedError::CatchError),
            Ok(ZEND_RESULT_CODE_SUCCESS) => Ok(result),
            Ok(_) => Err(EmbedError::ExecuteError(ExecutorGlobals::take_exception())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Embed;

    #[test]
    fn test_run() {
        Embed::run(|| {
            let result = Embed::eval("$foo = 'foo';");

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_run_error() {
        Embed::run(|| {
            let result = Embed::eval("stupid code;");

            assert!(!result.is_ok());
        });
    }

    #[test]
    fn test_run_script() {
        Embed::run(|| {
            let result = Embed::run_script("src/embed/test-script.php");

            assert!(result.is_ok());

            let zval = Embed::eval("$foo;").unwrap();

            assert!(zval.is_object());

            let obj = zval.object().unwrap();

            assert_eq!(obj.get_class_name().unwrap(), "Test");
        });
    }

    #[test]
    fn test_run_script_error() {
        Embed::run(|| {
            let result = Embed::run_script("src/embed/test-script-exception.php");

            assert!(!result.is_ok());
        });
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        Embed::run(|| {
            panic!("test panic");
        });
    }

    #[test]
    fn test_return() {
        let foo = Embed::run(|| {
            return "foo";
        });

        assert_eq!(foo, "foo");
    }

    #[test]
    fn test_eval_bailout() {
        Embed::run(|| {
            let result = Embed::eval("trigger_error(\"Fatal error\", E_USER_ERROR);");

            assert!(result.is_err());
            assert!(result.unwrap_err().is_bailout());
        });
    }
}
