use crate::convert::FromZvalMut;
use crate::ffi::zend_object_iterator;
use crate::flags::DataType;
use crate::types::{ZendLong, Zval};
use std::convert::TryInto;
use std::fmt::Display;

pub type ZendIterator = zend_object_iterator;

impl ZendIterator {
    pub fn iter(&mut self) -> Iter {
        self.index = 0;
        self.rewind();

        Iter { zi: self }
    }

    pub fn valid(&mut self) -> bool {
        if let Some(valid) = unsafe { (*self.funcs).valid } {
            unsafe { valid(&mut *self) != 0 }
        } else {
            true
        }
    }

    pub fn rewind(&mut self) {
        if let Some(rewind) = unsafe { (*self.funcs).rewind } {
            unsafe {
                rewind(&mut *self);
            }
        }
    }

    pub fn move_forward(&mut self) {
        if let Some(move_forward) = unsafe { (*self.funcs).move_forward } {
            unsafe {
                move_forward(&mut *self);
            }
        }
    }

    pub fn get_current_data<'a>(&mut self) -> Option<&'a Zval> {
        let get_current_data = unsafe { (*self.funcs).get_current_data }?;
        let value = unsafe { &*get_current_data(&mut *self) };

        Some(value)
    }

    pub fn get_current_key(&mut self) -> Option<Zval> {
        let get_current_key = unsafe { (*self.funcs).get_current_key }?;
        let mut key = Zval::new();
        unsafe {
            get_current_key(&mut *self, &mut key);
        }

        Some(key)
    }
}

#[derive(PartialEq)]
pub enum IterKey {
    Long(u64),
    String(String),
}

impl IterKey {
    pub fn is_numerical(&self) -> bool {
        match self {
            IterKey::Long(_) => true,
            IterKey::String(_) => false,
        }
    }
}

impl Display for IterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterKey::Long(key) => write!(f, "{}", key),
            IterKey::String(key) => write!(f, "{}", key),
        }
    }
}

pub struct Iter<'a> {
    zi: &'a mut ZendIterator,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (IterKey, &'a Zval);

    fn next(&mut self) -> Option<Self::Item> {
        // Call next when index > 0, so next is really called at the start of each iteration, which allow to work better with generator iterator
        if self.zi.index > 0 {
            self.zi.move_forward();

            if !self.zi.valid() {
                return None;
            }
        }

        self.zi.index += 1;

        let key = self.zi.get_current_key();
        let value = self.zi.get_current_data()?;
        let real_index = self.zi.index - 1;

        Some(match key {
            Some(key) => match key.is_long() {
                false => match key.try_into() {
                    Ok(key) => (IterKey::String(key), value),
                    Err(_) => (IterKey::Long(real_index), value),
                },
                true => (IterKey::Long(key.long().unwrap_or(real_index as ZendLong) as u64), value),
            },
            None => (IterKey::Long(real_index), value),
        })
    }
}

impl<'a> FromZvalMut<'a> for &'a mut ZendIterator {
    const TYPE: DataType = DataType::Object(Some("Traversable"));

    fn from_zval_mut(zval: &'a mut Zval) -> Option<Self> {
        let zend_object = zval.object()?;
        let ce = zend_object.get_class_entry_mut();
        let iterator = unsafe { ce.get_iterator?(&mut *ce, &mut *zval, 0) };

        unsafe { iterator.as_mut() }
    }
}
