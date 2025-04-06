//! Traits and types for interacting with reference counted PHP types.

use std::fmt::Debug;

use crate::{
    ffi::{zend_refcounted_h, zend_string},
    types::ZendObject,
};

/// Object used to store Zend reference counter.
pub type ZendRefcount = zend_refcounted_h;

impl Debug for ZendRefcount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZendRefcount")
            .field("refcount", &self.refcount)
            .finish()
    }
}

/// Implemented on refcounted types.
pub trait PhpRc {
    /// Returns an immutable reference to the corresponding refcount object.
    fn get_rc(&self) -> &ZendRefcount;

    /// Returns a mutable reference to the corresponding refcount object.
    fn get_rc_mut(&mut self) -> &mut ZendRefcount;

    /// Returns the number of references to the object.
    fn get_count(&self) -> u32 {
        self.get_rc().refcount
    }

    /// Increments the reference counter by 1.
    fn inc_count(&mut self) {
        self.get_rc_mut().refcount += 1;
    }

    /// Decrements the reference counter by 1.
    fn dec_count(&mut self) {
        self.get_rc_mut().refcount -= 1;
    }
}

macro_rules! rc {
    ($($t: ty),*) => {
        $(
            impl PhpRc for $t {
                fn get_rc(&self) -> &ZendRefcount {
                    &self.gc
                }

                fn get_rc_mut(&mut self) -> &mut ZendRefcount {
                    &mut self.gc
                }
            }
        )*
    };
}

rc!(ZendObject, zend_string);
