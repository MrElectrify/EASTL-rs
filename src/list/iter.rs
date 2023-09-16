use crate::list::node::{ListNode, ListNodeBase};
use std::marker::PhantomData;

/// Iterator over `eastl::List`, yielding references in the list's order
pub struct Iter<'a, T: 'a> {
    sentinel_node: *const ListNodeBase,
    current_node: *mut ListNodeBase,
    len: usize,
    marker: PhantomData<&'a ListNode<T>>,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(sentinel_node: *const ListNodeBase, len: usize) -> Self {
        Self {
            sentinel_node,
            current_node: sentinel_node.cast_mut(),
            len,
            marker: PhantomData,
        }
    }
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
    sentinel_node: *const ListNodeBase,
    current_node: *mut ListNodeBase,
    len: usize,
    marker: PhantomData<&'a mut ListNode<T>>,
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(sentinel_node: *const ListNodeBase, len: usize) -> Self {
        Self {
            sentinel_node,
            current_node: sentinel_node.cast_mut(),
            len,
            marker: PhantomData,
        }
    }
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
