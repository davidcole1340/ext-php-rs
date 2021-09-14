use crate::bindings::zend_eval_stringl;
use crate::errors::{Error, Result};
use crate::php::globals::ExecutorGlobals;
use crate::php::types::zval::Zval;
use std::ffi::CString;
use std::rc::Rc;

pub fn eval(code: &str) -> Result<Zval> {
    let mut ret = Zval::new();
    let code = CString::new(code)?;

    let result = unsafe {
        zend_eval_stringl(
            code.as_ptr(),
            code.as_bytes().len() as u64,
            &mut ret,
            CString::new("")?.as_ptr(),
        )
    };

    if result < 0 {
        Err(Error::CompileFailed)
    } else if let Some(exception) = ExecutorGlobals::take_exception() {
        Err(Error::UncaughtException(Rc::new(exception)))
    } else {
        Ok(ret)
    }
}
