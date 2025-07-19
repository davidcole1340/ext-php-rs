use std::{collections::HashMap, convert::TryFrom};

use crate::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    error::{Error, Result},
    flags::DataType,
    types::Zval,
};

use super::super::ZendHashTable;

// TODO: Generalize hasher
#[allow(clippy::implicit_hasher)]
impl<'a, V> TryFrom<&'a ZendHashTable> for HashMap<String, V>
where
    V: FromZval<'a>,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
        let mut hm = HashMap::with_capacity(value.len());

        for (key, val) in value {
            hm.insert(
                key.to_string(),
                V::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?,
            );
        }

        Ok(hm)
    }
}

impl<K, V> TryFrom<HashMap<K, V>> for ZBox<ZendHashTable>
where
    K: AsRef<str>,
    V: IntoZval,
{
    type Error = Error;

    fn try_from(value: HashMap<K, V>) -> Result<Self> {
        let mut ht = ZendHashTable::with_capacity(
            value.len().try_into().map_err(|_| Error::IntegerOverflow)?,
        );

        for (k, v) in value {
            ht.insert(k.as_ref(), v)?;
        }

        Ok(ht)
    }
}

// TODO: Generalize hasher
#[allow(clippy::implicit_hasher)]
impl<K, V> IntoZval for HashMap<K, V>
where
    K: AsRef<str>,
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

// TODO: Generalize hasher
#[allow(clippy::implicit_hasher)]
impl<'a, T> FromZval<'a> for HashMap<String, T>
where
    T: FromZval<'a>,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}
