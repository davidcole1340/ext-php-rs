use crate::errors::{Error, Result};

use super::{
    exceptions::PhpException,
    types::zval::{FromZval, IntoZval, Zval},
};

/// Implemented on types which can be used as PHP properties.
///
/// Generally, this should not be directly implemented on types, as it is automatically implemented on
/// types that implement [`Clone`], [`IntoZval`] and [`FromZval`], which will be required to implement
/// this trait regardless.
pub trait Prop<'zval> {
    /// Gets the value of `self` by setting the value of `zv`.
    ///
    /// # Parameters
    ///
    /// * `zv` - The zval to set the value of.
    fn get(&self, zv: &mut Zval) -> Result<()>;

    /// Sets the value of `self` with the contents of a given zval `zv`.
    ///
    /// # Parameters
    ///
    /// * `zv` - The zval containing the new value of `self`.
    fn set(&mut self, zv: &'zval Zval) -> Result<()>;
}

impl<'a, T: Clone + IntoZval + FromZval<'a>> Prop<'a> for T {
    fn get(&self, zv: &mut Zval) -> Result<()> {
        self.clone().set_zval(zv, false)
    }

    fn set(&mut self, zv: &'a Zval) -> Result<()> {
        let x = Self::from_zval(zv).ok_or(Error::ZvalConversion(zv.get_type()?))?;
        *self = x;
        Ok(())
    }
}

pub enum Property<'a, 'b, T> {
    Field(&'a mut dyn Prop<'a>),
    GetterSetter {
        get: Option<Box<dyn Fn(&'b T, &mut Zval) -> Option<()> + 'b>>,
        set: Option<Box<dyn FnMut(&'b mut T, Zval) -> Option<()> + 'b>>,
    },
}

impl<'a, 'b, T> Property<'a, 'b, T> {
    pub fn field(field: &'a mut dyn Prop<'a>) -> Self {
        Self::Field(field)
    }

    pub fn method<V>(get: Option<fn(&'b T) -> V>, set: Option<fn(&'b mut T, V)>) -> Self
    where
        V: IntoZval + From<Zval> + 'b,
    {
        let get = get.map(|get| {
            Box::new(move |self_: &'b T, zv: &mut Zval| {
                let result = get(self_);
                result.set_zval(zv, false).ok()?;
                Some(())
            }) as Box<dyn Fn(&'b T, &mut Zval) -> Option<()> + 'b>
        });

        let set = set.map(|set| {
            Box::new(move |self_: &'b mut T, zv: Zval| {
                let val = zv.into();
                set(self_, val);
                Some(())
            }) as Box<dyn FnMut(&'b mut T, Zval) -> Option<()> + 'b>
        });

        Self::GetterSetter { get, set }
    }

    pub fn get(&self, self_: &'b T, zv: &mut Zval) -> Result<(), PhpException<'static>> {
        match self {
            Property::Field(field) => field
                .get(zv)
                .map_err(|e| PhpException::from(format!("Failed to get property value: {}", e))),
            Property::GetterSetter { get, set: _ } => {
                if let Some(get) = get {
                    get(self_, zv)
                        .ok_or_else(|| PhpException::from("Failed to get property value."))
                } else {
                    Err(PhpException::from("This property has no get handler."))
                }
            }
        }
    }

    pub fn set(&mut self, self_: &'b mut T, zv: &Zval) -> Result<(), PhpException<'static>> {
        todo!()
        // match self {
        //     Property::Field(field) => field
        //         .set(zv)
        //         .map_err(|e| PhpException::from(format!("Failed to set property value: {}", e))),
        //     Property::GetterSetter { get: _, set } => {
        //         if let Some(set) = set {
        //             set(self_, zv)
        //                 .ok_or_else(|| PhpException::from("Failed to set property value."))
        //         } else {
        //             Err(PhpException::from("This property has no set handler."))
        //         }
        //     }
        // }
    }
}
