use crate::{convert::FromZval, error::Error, flags::DataType, types::Zval};
use std::str::FromStr;
use std::{convert::TryFrom, fmt::Display};

/// Represents the key of a PHP array, which can be either a long or a string.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayKey<'a> {
    /// A numerical key.
    /// In Zend API it's represented by `u64` (`zend_ulong`), so the value needs
    /// to be cast to `zend_ulong` before passing into Zend functions.
    Long(i64),
    /// A string key.
    String(String),
    /// A string key by reference.
    Str(&'a str),
}

impl From<String> for ArrayKey<'_> {
    fn from(value: String) -> Self {
        if let Ok(index) = i64::from_str(value.as_str()) {
            Self::Long(index)
        } else {
            Self::String(value)
        }
    }
}

impl TryFrom<ArrayKey<'_>> for String {
    type Error = Error;

    fn try_from(value: ArrayKey<'_>) -> Result<Self, Self::Error> {
        match value {
            ArrayKey::String(s) => Ok(s),
            ArrayKey::Str(s) => Ok(s.to_string()),
            ArrayKey::Long(l) => Ok(l.to_string()),
        }
    }
}

impl TryFrom<ArrayKey<'_>> for i64 {
    type Error = Error;

    fn try_from(value: ArrayKey<'_>) -> Result<Self, Self::Error> {
        match value {
            ArrayKey::Long(i) => Ok(i),
            ArrayKey::String(s) => s.parse::<i64>().map_err(|_| Error::InvalidProperty),
            ArrayKey::Str(s) => s.parse::<i64>().map_err(|_| Error::InvalidProperty),
        }
    }
}

impl ArrayKey<'_> {
    /// Check if the key is an integer.
    ///
    /// # Returns
    ///
    /// Returns true if the key is an integer, false otherwise.
    #[must_use]
    pub fn is_long(&self) -> bool {
        match self {
            ArrayKey::Long(_) => true,
            ArrayKey::String(_) | ArrayKey::Str(_) => false,
        }
    }
}

impl Display for ArrayKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayKey::Long(key) => write!(f, "{key}"),
            ArrayKey::String(key) => write!(f, "{key}"),
            ArrayKey::Str(key) => write!(f, "{key}"),
        }
    }
}

impl<'a> From<&'a str> for ArrayKey<'a> {
    fn from(value: &'a str) -> ArrayKey<'a> {
        if let Ok(index) = i64::from_str(value) {
            Self::Long(index)
        } else {
            ArrayKey::Str(value)
        }
    }
}

impl<'a> From<i64> for ArrayKey<'a> {
    fn from(index: i64) -> ArrayKey<'a> {
        ArrayKey::Long(index)
    }
}

impl<'a> FromZval<'a> for ArrayKey<'_> {
    const TYPE: DataType = DataType::String;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        if let Some(key) = zval.long() {
            return Some(ArrayKey::Long(key));
        }
        if let Some(key) = zval.string() {
            return Some(ArrayKey::String(key));
        }
        None
    }
}

#[cfg(test)]
#[cfg(feature = "embed")]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::error::Error;
    use crate::types::ArrayKey;

    #[test]
    fn test_string_try_from_array_key() {
        let key = ArrayKey::String("test".to_string());
        let result: crate::error::Result<String, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test".to_string());

        let key = ArrayKey::Str("test");
        let result: crate::error::Result<String, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test".to_string());

        let key = ArrayKey::Long(42);
        let result: crate::error::Result<String, _> = key.try_into();
        assert_eq!(result.unwrap(), "42".to_string());

        let key = ArrayKey::String("42".to_string());
        let result: crate::error::Result<String, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42".to_string());

        let key = ArrayKey::Str("123");
        let result: crate::error::Result<i64, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn test_i64_try_from_array_key() {
        let key = ArrayKey::Long(42);
        let result: crate::error::Result<i64, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let key = ArrayKey::String("42".to_string());
        let result: crate::error::Result<i64, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let key = ArrayKey::Str("123");
        let result: crate::error::Result<i64, _> = key.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);

        let key = ArrayKey::String("not a number".to_string());
        let result: crate::error::Result<i64, _> = key.try_into();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidProperty));
    }
}
