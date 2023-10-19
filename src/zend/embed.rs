use crate::boxed::ZBox;
use crate::ffi::{php_embed_init, php_embed_shutdown, zend_eval_stringl, ZEND_RESULT_CODE_SUCCESS};
use crate::types::{ZendObject, Zval};
use crate::zend::ExecutorGlobals;
use parking_lot::{const_rwlock, RwLock, RwLockWriteGuard};
use std::ffi::{c_char, CString};
use std::ops::Deref;
use std::ptr::null_mut;

pub struct Embed;

#[derive(Debug)]
pub enum EmbedError {
    InitError,
    ExecuteError(Option<ZBox<ZendObject>>),
}

static EMBED_LOCK: RwLock<bool> = const_rwlock(false);

pub struct EmbedWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, bool>,
    data: T,
}

impl<'a, T> Deref for EmbedWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Embed {
    fn init<'a>() -> EmbedWriteGuard<'a, Result<(), EmbedError>> {
        let mut write_guard = EMBED_LOCK.write();

        if !*write_guard {
            let result = unsafe { php_embed_init(0, null_mut()) };

            if result != ZEND_RESULT_CODE_SUCCESS {
                return EmbedWriteGuard {
                    guard: write_guard,
                    data: Err(EmbedError::InitError),
                };
            }

            *write_guard = true
        }

        return EmbedWriteGuard {
            guard: write_guard,
            data: Ok(()),
        };
    }

    // We use a write guard, this should only be called once at a time in non thread safe
    // In the future we may not need this when it's linked to a thread safe php
    pub fn run<'a>(code: &str) -> EmbedWriteGuard<'a, Result<Zval, EmbedError>> {
        let guard = Self::init();

        if guard.data.is_err() {
            return EmbedWriteGuard {
                guard: guard.guard,
                data: Err(EmbedError::InitError),
            };
        }

        let cstr = CString::new(code).unwrap();
        let mut result = Zval::new();

        // this eval is very limited as it only allow simple code, it's the same eval used by php -r
        let exec_result = unsafe {
            zend_eval_stringl(
                cstr.as_ptr() as *const c_char,
                code.len() as _,
                &mut result,
                b"run\0".as_ptr() as *const _,
            )
        };

        let exception = ExecutorGlobals::take_exception();

        EmbedWriteGuard {
            guard: guard.guard,
            data: if exec_result != ZEND_RESULT_CODE_SUCCESS {
                Err(EmbedError::ExecuteError(exception))
            } else {
                Ok(result)
            },
        }
    }
}

impl Drop for Embed {
    fn drop(&mut self) {
        unsafe {
            php_embed_shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Embed;

    #[test]
    fn test_run() {
        let result = Embed::run("'foo';");

        assert!(result.is_ok());
    }

    #[test]
    fn test_run_error() {
        let result = Embed::run("stupid code");

        assert!(!result.is_ok());
    }
}
