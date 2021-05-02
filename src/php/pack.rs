//! Provides implementations for converting to and from Zend binary strings, commonly returned
//! from functions such as [`pack`] and [`unpack`].
//!
//! [`pack`]: https://www.php.net/manual/en/function.pack.php
//! [`unpack`]: https://www.php.net/manual/en/function.unpack.php

use super::types::string::ZendString;
use crate::bindings::ext_php_rs_zend_string_init;

/// Used to convert between Zend binary strings and vectors. Useful in conjunction with the
/// [`pack`] and [`unpack`] functions built-in to PHP.
///
/// # Safety
///
/// The types cannot be ensured between PHP and Rust, as the data is represented as a string when
/// crossing the language boundary. Exercise caution when using these functions.
///
/// [`pack`]: https://www.php.net/manual/en/function.pack.php
/// [`unpack`]: https://www.php.net/manual/en/function.unpack.php
pub unsafe trait Pack: Sized {
    /// Packs a given vector into a Zend binary string. Can be passed to PHP and then unpacked
    /// using the [`unpack`] function. Note you should probably use the [`set_binary`] method on the
    /// [`Zval`] struct instead of this function directly, as there is currently no way to set a
    /// [`ZendString`] on a [`Zval`] directly.
    ///
    /// # Parameters
    ///
    /// * `vec` - The vector to pack into a binary string.
    ///
    /// [`unpack`]: https://www.php.net/manual/en/function.unpack.php
    /// [`Zval`]: crate::php::types::zval::Zval
    /// [`ZendString`]: crate::php::types::string::ZendString
    /// [`set_binary`]: crate::php::types::zval::Zval#method.set_binary
    fn pack_into(vec: Vec<Self>) -> *mut ZendString;

    /// Unpacks a given Zend binary string into a Rust vector. Can be used to pass data from `pack`
    /// in PHP to Rust without encoding into another format. Note that the data *must* be all one
    /// type, as this implementation only unpacks one type.
    ///
    /// # Safety
    ///
    /// This is an unsafe function. There is no way to tell if the data passed from the PHP
    /// function is indeed the correct format. Exercise caution when using the `unpack` functions.
    /// In fact, even when used correctly, the results can differ depending on the platform and the
    /// size of each type on the platform. Consult the [`pack`](https://www.php.net/manual/en/function.pack.php)
    /// function documentation for more details.
    ///
    /// # Parameters
    ///
    /// * `s` - The Zend string containing the binary data.
    unsafe fn unpack_into(s: &ZendString) -> Vec<Self>;
}

/// Implements the [`Pack`] trait for a given type.
/// The first argument is the type and the second argument is the factor of size difference between
/// the given type and an 8-bit integer e.g. impl Unpack for i32, factor = 4 => 4 * 8 = 32
#[macro_use]
macro_rules! pack_impl {
    ($t: ty, $d: expr) => {
        unsafe impl Pack for $t {
            fn pack_into(vec: Vec<Self>) -> *mut ZendString {
                let len = vec.len() * $d;
                let ptr = Box::into_raw(vec.into_boxed_slice());
                unsafe { ext_php_rs_zend_string_init(ptr as *mut i8, len as _, false) }
            }

            unsafe fn unpack_into(s: &ZendString) -> Vec<Self> {
                let len = s.len / $d;
                let mut result = Vec::with_capacity(len as _);
                let ptr = s.val.as_ptr() as *const $t;

                for i in 0..len {
                    result.push(*ptr.offset(i as _));
                }

                result
            }
        }
    };
}

pack_impl!(u8, 1);
pack_impl!(i8, 1);

pack_impl!(u16, 2);
pack_impl!(i16, 2);

pack_impl!(u32, 4);
pack_impl!(i32, 4);

pack_impl!(u64, 8);
pack_impl!(i64, 8);

pack_impl!(f32, 4);
pack_impl!(f64, 8);
