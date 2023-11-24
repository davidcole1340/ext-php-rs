use std::marker::PhantomData;

use crate::ffi::{zend_llist, zend_llist_element, zend_llist_get_next_ex};

pub type ZendLinkedList = zend_llist;

impl ZendLinkedList {
    pub fn iter<T>(&self) -> ZendLinkedListIterator<T> {
        ZendLinkedListIterator::new(self)
    }
}

pub struct ZendLinkedListIterator<'a, T> {
    list: &'a zend_llist,
    position: *mut zend_llist_element,
    _marker: PhantomData<T>,
}

impl<'a, T> ZendLinkedListIterator<'a, T> {
    fn new(list: &'a ZendLinkedList) -> Self {
        ZendLinkedListIterator {
            list,
            position: list.head,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for ZendLinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.is_null() {
            return None;
        }
        let ptr = unsafe { (*self.position).data.as_mut_ptr() };
        let value = unsafe { &*(ptr as *const T as *mut T) };
        unsafe {
            zend_llist_get_next_ex(
                self.list as *const ZendLinkedList as *mut ZendLinkedList,
                &mut self.position,
            )
        };
        Some(value)
    }
}
