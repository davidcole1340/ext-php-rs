use crate::ffi::{
    ext_php_rs_zend_bailout, ext_php_rs_zend_first_try_catch, ext_php_rs_zend_try_catch,
};
use std::ffi::c_void;
use std::panic::{catch_unwind, resume_unwind, RefUnwindSafe};
use std::ptr::null_mut;

#[derive(Debug)]
pub struct CatchError;

pub(crate) unsafe extern "C" fn panic_wrapper<R, F: FnMut() -> R + RefUnwindSafe>(
    ctx: *const c_void,
) -> *const c_void {
    // we try to catch panic here so we correctly shutdown php if it happens
    // mandatory when we do assert on test as other test would not run correctly
    let panic = catch_unwind(|| (*(ctx as *mut F))());

    Box::into_raw(Box::new(panic)) as *mut c_void
}

/// PHP propose a try catch mechanism in C using setjmp and longjmp (bailout)
/// It store the arg of setjmp into the bailout field of the global executor
/// If a bailout is triggered, the executor will jump to the setjmp and restore the previous setjmp
///
/// try_catch allow to use this mechanism
///
/// # Returns
///
/// * `Ok(R)` - The result of the function
/// * `Err(CatchError)` - A bailout occurred during the execution
pub fn try_catch<R, F: FnMut() -> R + RefUnwindSafe>(
    func: F,
    first: bool,
) -> Result<R, CatchError> {
    let mut panic_ptr = null_mut();
    let has_bailout = unsafe {
        if first {
            ext_php_rs_zend_first_try_catch(
                panic_wrapper::<R, F>,
                &func as *const F as *const c_void,
                (&mut panic_ptr) as *mut *mut c_void,
            )
        } else {
            ext_php_rs_zend_try_catch(
                panic_wrapper::<R, F>,
                &func as *const F as *const c_void,
                (&mut panic_ptr) as *mut *mut c_void,
            )
        }
    };

    let panic = panic_ptr as *mut std::thread::Result<R>;

    // can be null if there is a bailout
    if panic.is_null() || has_bailout {
        return Err(CatchError);
    }

    match unsafe { *Box::from_raw(panic as *mut std::thread::Result<R>) } {
        Ok(r) => Ok(r),
        Err(err) => {
            // we resume the panic here so it can be catched correctly by the test framework
            resume_unwind(err);
        }
    }
}

/// Trigger a bailout
///
/// This function will stop the execution of the current script
/// and jump to the last try catch block
///
/// # Safety
///
/// This function is unsafe because it can cause memory leaks
/// Since it will jump to the last try catch block, it will not call the destructor of the current scope
///
/// When using this function you should ensure that all the memory allocated in the current scope is released
///
pub unsafe fn bailout() -> ! {
    ext_php_rs_zend_bailout();
}

#[cfg(feature = "embed")]
#[cfg(test)]
mod tests {
    use crate::embed::Embed;
    use crate::zend::{bailout, try_catch};
    use std::ptr::null_mut;

    #[test]
    fn test_catch() {
        Embed::run(|| {
            let catch = try_catch(
                || {
                    unsafe {
                        bailout();
                    }

                    #[allow(unreachable_code)]
                    {
                        assert!(false);
                    }
                },
                false,
            );

            assert!(catch.is_err());
        });
    }

    #[test]
    fn test_no_catch() {
        Embed::run(|| {
            let catch = try_catch(
                || {
                    assert!(true);
                },
                false,
            );

            assert!(catch.is_ok());
        });
    }

    #[test]
    fn test_bailout() {
        Embed::run(|| {
            unsafe {
                bailout();
            }

            #[allow(unreachable_code)]
            {
                assert!(false);
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        Embed::run(|| {
            let _ = try_catch(
                || {
                    panic!("should panic");
                },
                false,
            );
        });
    }

    #[test]
    fn test_return() {
        let foo = Embed::run(|| {
            let result = try_catch(
                || {
                    return "foo";
                },
                false,
            );

            assert!(result.is_ok());

            result.unwrap()
        });

        assert_eq!(foo, "foo");
    }

    #[test]
    fn test_memory_leak() {
        let mut ptr = null_mut();

        let _ = try_catch(
            || {
                let mut result = "foo".to_string();
                ptr = &mut result;

                unsafe {
                    bailout();
                }
            },
            false,
        );

        // Check that the string is never released
        let result = unsafe { &*ptr as &str };

        assert_eq!(result, "foo");
    }
}
