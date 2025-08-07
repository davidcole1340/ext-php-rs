use std::{collections::BTreeMap, convert::TryFrom};

use super::super::ZendHashTable;
use crate::types::ArrayKey;
use crate::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    error::{Error, Result},
    flags::DataType,
    types::Zval,
};

impl<'a, K, V> TryFrom<&'a ZendHashTable> for BTreeMap<K, V>
where
    K: TryFrom<ArrayKey<'a>, Error = Error> + Ord,
    V: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
        let mut map = Self::new();

        for (key, val) in value {
            map.insert(
                key.try_into()?,
                V::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?,
            );
        }

        Ok(map)
    }
}

impl<'a, V> TryFrom<&'a ZendHashTable> for BTreeMap<ArrayKey<'a>, V>
where
    V: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
        let mut map = Self::new();

        for (key, val) in value {
            map.insert(
                key,
                V::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?,
            );
        }

        Ok(map)
    }
}

impl<'a, K, V> TryFrom<BTreeMap<K, V>> for ZBox<ZendHashTable>
where
    K: Into<ArrayKey<'a>>,
    V: IntoZval,
{
    type Error = Error;

    fn try_from(value: BTreeMap<K, V>) -> Result<Self> {
        let mut ht = ZendHashTable::with_capacity(
            value.len().try_into().map_err(|_| Error::IntegerOverflow)?,
        );

        for (k, v) in value {
            ht.insert(k, v)?;
        }

        Ok(ht)
    }
}

impl<'a, K, V> IntoZval for BTreeMap<K, V>
where
    K: Into<ArrayKey<'a>>,
    V: IntoZval,
{
    const TYPE: DataType = DataType::Array;
    const NULLABLE: bool = false;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let arr = self.try_into()?;
        zv.set_hashtable(arr);
        Ok(())
    }
}

impl<'a, K, V> FromZval<'a> for BTreeMap<K, V>
where
    K: TryFrom<ArrayKey<'a>, Error = Error> + Ord,
    V: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}

impl<'a, V> FromZval<'a> for BTreeMap<ArrayKey<'a>, V>
where
    V: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}
