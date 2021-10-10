//! Types defined by the Zend engine used in PHP.
//!
//! Generally, it is easier to work directly with Rust types, converting into
//! these PHP types when required.

mod array;
mod callable;
mod class_object;
mod long;
mod object;
mod string;
mod zval;

pub use array::ZendHashTable;
pub use callable::ZendCallable;
pub use class_object::ZendClassObject;
pub use long::ZendLong;
pub use object::{PropertyQuery, ZendObject};
pub use string::ZendStr;
pub use zval::Zval;

use crate::{convert::FromZval, flags::DataType, macros::into_zval};

into_zval!(f32, set_double, Double);
into_zval!(f64, set_double, Double);
into_zval!(bool, set_bool, Bool);

try_from_zval!(f64, double, Double);
try_from_zval!(bool, bool, Bool);

impl FromZval<'_> for f32 {
    const TYPE: DataType = DataType::Double;

    fn from_zval(zval: &Zval) -> Option<Self> {
        zval.double().map(|v| v as f32)
    }
}
