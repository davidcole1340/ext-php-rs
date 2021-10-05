//! Provides implementations for converting to and from Zend binary strings, commonly returned
//! from functions such as [`pack`] and [`unpack`].
//!
//! [`pack`]: https://www.php.net/manual/en/function.pack.php
//! [`unpack`]: https://www.php.net/manual/en/function.unpack.php

use crate::bindings::{ext_php_rs_zend_string_init, zend_string};

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
pub unsafe trait Pack: Clone {
    /// Packs a given vector into a Zend binary string. Can be passed to PHP and then unpacked
    /// using the [`unpack`] function.
    ///
    /// # Parameters
    ///
    /// * `vec` - The vector to pack into a binary string.
    ///
    /// [`unpack`]: https://www.php.net/manual/en/function.unpack.php
    fn pack_into(vec: Vec<Self>) -> *mut zend_string;

    /// Unpacks a given Zend binary string into a Rust vector. Can be used to pass data from `pack`
    /// in PHP to Rust without encoding into another format. Note that the data *must* be all one
    /// type, as this implementation only unpacks one type.
    ///
    /// # Safety
    ///
    /// There is no way to tell if the data stored in the string is actually of the given type.
    /// The results of this function can also differ from platform-to-platform due to the different
    /// representation of some types on different platforms. Consult the [`pack`] function
    /// documentation for more details.
    ///
    /// # Parameters
    ///
    /// * `s` - The Zend string containing the binary data.
    ///
    /// [`pack`]: https://www.php.net/manual/en/function.pack.php
    fn unpack_into(s: &zend_string) -> Vec<Self>;
}

/// Implements the [`Pack`] trait for a given type.
macro_rules! pack_impl {
    ($t: ty) => {
        pack_impl!($t, <$t>::BITS);
    };

    ($t: ty, $d: expr) => {
        unsafe impl Pack for $t {
            fn pack_into(vec: Vec<Self>) -> *mut zend_string {
                let len = vec.len() * ($d as usize / 8);
                let ptr = Box::into_raw(vec.into_boxed_slice());
                unsafe { ext_php_rs_zend_string_init(ptr as *mut i8, len as _, false) }
            }

            fn unpack_into(s: &zend_string) -> Vec<Self> {
                let bytes = ($d / 8) as u64;
                let len = (s.len as u64) / bytes;
                let mut result = Vec::with_capacity(len as _);
                let ptr = s.val.as_ptr() as *const $t;

                // SAFETY: We calculate the length of memory that we can legally read based on the
                // side of the type, therefore we never read outside the memory we should.
                for i in 0..len {
                    result.push(unsafe { *ptr.offset(i as _) });
                }

                result
            }
        }
    };
}

pack_impl!(u8);
pack_impl!(i8);

pack_impl!(u16);
pack_impl!(i16);

pack_impl!(u32);
pack_impl!(i32);

pack_impl!(u64);
pack_impl!(i64);

pack_impl!(isize);
pack_impl!(usize);

pack_impl!(f32, 32);
pack_impl!(f64, 64);
