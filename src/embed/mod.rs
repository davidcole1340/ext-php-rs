mod ffi;

use crate::boxed::ZBox;
use crate::embed::ffi::ext_php_rs_embed_callback;
use crate::ffi::{
    _zend_file_handle__bindgen_ty_1, php_execute_script, zend_eval_string, zend_file_handle,
    zend_stream_init_filename, ZEND_RESULT_CODE_SUCCESS,
};
use crate::types::{ZendObject, Zval};
use crate::zend::ExecutorGlobals;
use parking_lot::{const_rwlock, RwLock};
use std::ffi::{c_char, c_void, CString};
use std::path::Path;
use std::ptr::null_mut;

pub struct Embed;

#[derive(Debug)]
pub enum EmbedError {
    InitError,
    ExecuteError(Option<ZBox<ZendObject>>),
}

static RUN_FN_LOCK: RwLock<()> = const_rwlock(());

impl Embed {
    pub fn run_script<P: AsRef<Path>>(path: P) -> bool {
        let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
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

        unsafe { php_execute_script(&mut file_handle) }
    }

    pub fn run<F: Fn()>(func: F) {
        // @TODO handle php thread safe
        //
        // This is to prevent multiple threads from running php at the same time
        // At some point we should detect if php is compiled with thread safety and avoid doing that in this case
        let _guard = RUN_FN_LOCK.write();

        unsafe extern "C" fn wrapper<F: Fn()>(ctx: *const c_void) {
            (*(ctx as *const F))();
        }

        unsafe {
            ext_php_rs_embed_callback(
                0,
                null_mut(),
                wrapper::<F>,
                &func as *const F as *const c_void,
            );
        }
    }

    pub fn eval(code: &str) -> Result<Zval, EmbedError> {
        let cstr = CString::new(code).unwrap();
        let mut result = Zval::new();

        // this eval is very limited as it only allow simple code, it's the same eval used by php -r
        let exec_result = unsafe {
            zend_eval_string(
                cstr.as_ptr() as *const c_char,
                &mut result,
                b"run\0".as_ptr() as *const _,
            )
        };

        let exception = ExecutorGlobals::take_exception();

        if exec_result != ZEND_RESULT_CODE_SUCCESS {
            Err(EmbedError::ExecuteError(exception))
        } else {
            Ok(result)
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

            assert!(result);

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

            assert!(!result);
        });
    }
}
