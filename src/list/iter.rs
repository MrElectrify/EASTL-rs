use crate::list::node::{ListNode, ListNodeBase};
use std::marker::PhantomData;

/// Iterator over `eastl::List`, yielding references in the list's order
pub struct Iter<'a, T: 'a> {
    pub(crate) len: usize,
    pub(crate) sentinel_node: *const ListNodeBase<T>,
    pub(crate) current_node: *mut ListNodeBase<T>,
    pub(crate) marker: PhantomData<&'a ListNodeBase<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { (*self.current_node).next.cast_const() } == self.sentinel_node {
            None
        } else {
            self.len -= 1;
            self.current_node = unsafe { (*self.current_node).next };
            let node = self.current_node as *const ListNode<T>;
            Some(unsafe { (*node).value() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

/// Iterator over `eastl::List`, yielding mutable references in the list's order
pub struct IterMut<'a, T: 'a> {
    pub(crate) len: usize,
    pub(crate) sentinel_node: *const ListNodeBase<T>,
    pub(crate) current_node: *mut ListNodeBase<T>,
    pub(crate) marker: PhantomData<&'a mut ListNodeBase<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { (*self.current_node).next.cast_const() } == self.sentinel_node {
            None
        } else {
            self.len -= 1;
            self.current_node = unsafe { (*self.current_node).next };
            let node = self.current_node as *mut ListNode<T>;
            Some(unsafe { (*node).value_mut() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
