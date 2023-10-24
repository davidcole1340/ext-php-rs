use std::{ffi::c_void, mem::MaybeUninit, os::raw::c_int, ptr};

use crate::{
    class::RegisteredClass,
    exception::PhpResult,
    ffi::{
        std_object_handlers, zend_is_true, zend_object_handlers, zend_object_std_dtor,
        zend_std_get_properties, zend_std_has_property, zend_std_read_property,
        zend_std_write_property,
    },
    flags::ZvalTypeFlags,
    types::{ZendClassObject, ZendHashTable, ZendObject, ZendStr, Zval},
};

/// A set of functions associated with a PHP class.
pub type ZendObjectHandlers = zend_object_handlers;

impl ZendObjectHandlers {
    /// Creates a new set of object handlers based on the standard object
    /// handlers.
    pub fn new<T: RegisteredClass>() -> ZendObjectHandlers {
        let mut this = MaybeUninit::uninit();

        // SAFETY: `this` is allocated on the stack and is a valid memory location.
        unsafe { Self::init::<T>(&mut *this.as_mut_ptr()) };

        // SAFETY: We just initialized the handlers in the previous statement, therefore
        // we are returning a valid object.
        unsafe { this.assume_init() }
    }

    /// Initializes a given set of object handlers by copying the standard
    /// object handlers into the memory location, as well as setting up the
    /// `T` type destructor.
    ///
    /// # Parameters
    ///
    /// * `ptr` - Pointer to memory location to copy the standard handlers to.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that the `ptr` given is a valid memory location.
    pub unsafe fn init<T: RegisteredClass>(ptr: *mut ZendObjectHandlers) {
        std::ptr::copy_nonoverlapping(&std_object_handlers, ptr, 1);
        let offset = ZendClassObject::<T>::std_offset();
        (*ptr).offset = offset as _;
        (*ptr).free_obj = Some(Self::free_obj::<T>);
        (*ptr).read_property = Some(Self::read_property::<T>);
        (*ptr).write_property = Some(Self::write_property::<T>);
        (*ptr).get_properties = Some(Self::get_properties::<T>);
        (*ptr).has_property = Some(Self::has_property::<T>);
    }

    unsafe extern "C" fn free_obj<T: RegisteredClass>(object: *mut ZendObject) {
        object
            .as_mut()
            .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
            .map(|obj|ptr::drop_in_place(&mut obj.obj));

        // Manually drop the object as we don't want to free the underlying memory.


        zend_object_std_dtor(object)
    }

    unsafe extern "C" fn read_property<T: RegisteredClass>(
        object: *mut ZendObject,
        member: *mut ZendStr,
        type_: c_int,
        cache_slot: *mut *mut c_void,
        rv: *mut Zval,
    ) -> *mut Zval {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut ZendObject,
            member: *mut ZendStr,
            type_: c_int,
            cache_slot: *mut *mut c_void,
            rv: *mut Zval,
        ) -> PhpResult<*mut Zval> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = &mut **obj;
            let props = T::get_metadata().get_properties();
            let prop = props.get(prop_name.as_str()?);

            // retval needs to be treated as initialized, so we set the type to null
            let rv_mut = rv.as_mut().ok_or("Invalid return zval given")?;
            rv_mut.u1.type_info = ZvalTypeFlags::Null.bits();

            Ok(match prop {
                Some(prop) => {
                    prop.get(self_, rv_mut)?;
                    rv
                }
                None => zend_std_read_property(object, member, type_, cache_slot, rv),
            })
        }

        match internal::<T>(object, member, type_, cache_slot, rv) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                (*rv).set_null();
                rv
            }
        }
    }

    unsafe extern "C" fn write_property<T: RegisteredClass>(
        object: *mut ZendObject,
        member: *mut ZendStr,
        value: *mut Zval,
        cache_slot: *mut *mut c_void,
    ) -> *mut Zval {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut ZendObject,
            member: *mut ZendStr,
            value: *mut Zval,
            cache_slot: *mut *mut c_void,
        ) -> PhpResult<*mut Zval> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let self_ = &mut **obj;
            let props = T::get_metadata().get_properties();
            let prop = props.get(prop_name.as_str()?);
            let value_mut = value.as_mut().ok_or("Invalid return zval given")?;

            Ok(match prop {
                Some(prop) => {
                    prop.set(self_, value_mut)?;
                    value
                }
                None => zend_std_write_property(object, member, value, cache_slot),
            })
        }

        match internal::<T>(object, member, value, cache_slot) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                value
            }
        }
    }

    unsafe extern "C" fn get_properties<T: RegisteredClass>(
        object: *mut ZendObject,
    ) -> *mut ZendHashTable {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut ZendObject,
            props: &mut ZendHashTable,
        ) -> PhpResult {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let self_ = &mut **obj;
            let struct_props = T::get_metadata().get_properties();

            for (name, val) in struct_props {
                let mut zv = Zval::new();
                if val.get(self_, &mut zv).is_err() {
                    continue;
                }
                props.insert(name, zv).map_err(|e| {
                    format!("Failed to insert value into properties hashtable: {e:?}")
                })?;
            }

            Ok(())
        }

        let props = zend_std_get_properties(object)
            .as_mut()
            .or_else(|| Some(ZendHashTable::new().into_raw()))
            .expect("Failed to get property hashtable");

        if let Err(e) = internal::<T>(object, props) {
            let _ = e.throw();
        }

        props
    }

    unsafe extern "C" fn has_property<T: RegisteredClass>(
        object: *mut ZendObject,
        member: *mut ZendStr,
        has_set_exists: c_int,
        cache_slot: *mut *mut c_void,
    ) -> c_int {
        #[inline(always)]
        unsafe fn internal<T: RegisteredClass>(
            object: *mut ZendObject,
            member: *mut ZendStr,
            has_set_exists: c_int,
            cache_slot: *mut *mut c_void,
        ) -> PhpResult<c_int> {
            let obj = object
                .as_mut()
                .and_then(|obj| ZendClassObject::<T>::from_zend_obj_mut(obj))
                .ok_or("Invalid object pointer given")?;
            let prop_name = member
                .as_ref()
                .ok_or("Invalid property name pointer given")?;
            let props = T::get_metadata().get_properties();
            let prop = props.get(prop_name.as_str()?);
            let self_ = &mut **obj;

            match has_set_exists {
                //
                // * 0 (has) whether property exists and is not NULL
                0 => {
                    if let Some(val) = prop {
                        let mut zv = Zval::new();
                        val.get(self_, &mut zv)?;
                        if !zv.is_null() {
                            return Ok(1);
                        }
                    }
                }
                //
                // * 1 (set) whether property exists and is true
                1 => {
                    if let Some(val) = prop {
                        let mut zv = Zval::new();
                        val.get(self_, &mut zv)?;

                        if zend_is_true(&mut zv) == 1 {
                            return Ok(1);
                        }
                    }
                }
                //
                // * 2 (exists) whether property exists
                2 => {
                    if prop.is_some() {
                        return Ok(1);
                    }
                }
                _ => return Err(
                    "Invalid value given for `has_set_exists` in struct `has_property` function."
                        .into(),
                ),
            };

            Ok(zend_std_has_property(
                object,
                member,
                has_set_exists,
                cache_slot,
            ))
        }

        match internal::<T>(object, member, has_set_exists, cache_slot) {
            Ok(rv) => rv,
            Err(e) => {
                let _ = e.throw();
                0
            }
        }
    }
}
