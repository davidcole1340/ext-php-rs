use super::super::ZendHashTable;
use crate::types::ArrayKey;
use crate::{
    boxed::ZBox,
    convert::{FromZval, IntoZval},
    error::{Error, Result},
    flags::DataType,
    types::Zval,
};
use std::hash::{BuildHasher, Hash};
use std::{collections::HashMap, convert::TryFrom};

impl<'a, K, V, H> TryFrom<&'a ZendHashTable> for HashMap<K, V, H>
where
    K: TryFrom<ArrayKey<'a>, Error = Error> + Eq + Hash,
    V: FromZval<'a>,
    H: BuildHasher + Default,
{
    type Error = Error;

    fn try_from(value: &'a ZendHashTable) -> Result<Self> {
        let mut hm = Self::with_capacity_and_hasher(value.len(), H::default());

        for (key, val) in value {
            hm.insert(
                key.try_into()?,
                V::from_zval(val).ok_or_else(|| Error::ZvalConversion(val.get_type()))?,
            );
        }

        Ok(hm)
    }
}

impl<K, V, H> TryFrom<HashMap<K, V, H>> for ZBox<ZendHashTable>
where
    K: AsRef<str>,
    V: IntoZval,
    H: BuildHasher,
{
    type Error = Error;

    fn try_from(value: HashMap<K, V, H>) -> Result<Self> {
        let mut ht = ZendHashTable::with_capacity(
            value.len().try_into().map_err(|_| Error::IntegerOverflow)?,
        );

        for (k, v) in value {
            ht.insert(k.as_ref(), v)?;
        }

        Ok(ht)
    }
}

impl<K, V, H> IntoZval for HashMap<K, V, H>
where
    K: AsRef<str>,
    V: IntoZval,
    H: BuildHasher,
{
    const TYPE: DataType = DataType::Array;
    const NULLABLE: bool = false;

    fn set_zval(self, zv: &mut Zval, _: bool) -> Result<()> {
        let arr = self.try_into()?;
        zv.set_hashtable(arr);
        Ok(())
    }
}

impl<'a, V, H> FromZval<'a> for HashMap<String, V, H>
where
    V: FromZval<'a>,
    H: BuildHasher + Default,
{
    const TYPE: DataType = DataType::Array;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.array().and_then(|arr| arr.try_into().ok())
    }
}
