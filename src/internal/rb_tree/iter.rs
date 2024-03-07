use crate::internal::rb_tree::node::Node;
use std::marker::PhantomData;
use std::ptr;

/// An iterator over a Red-Black tree's nodes.
pub struct Iter<'a, K, V> {
    pub(super) node: *const Node<K, V>,
    pub(super) anchor: *const (),
    pub(super) _marker: PhantomData<&'a ()>,
}

pub struct IterMut<'a, K, V> {
    pub(super) node: *mut Node<K, V>,
    pub(super) anchor: *const (),
    pub(super) _marker: PhantomData<&'a mut ()>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if ptr::eq(self.node, self.anchor as *const _) {
            return None;
        }

        unsafe { self.node.as_ref() }
            .and_then(Node::next)
            .map(|node| {
                // update the iterator
                self.node = node;
                (node.key(), node.val())
            })
    }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if ptr::eq(self.node, self.anchor as *const _) {
            return None;
        }

        unsafe { self.node.as_mut() }
            .and_then(Node::next_mut)
            .map(|node| {
                // update the iterator
                self.node = node;
                (&node.pair.0, &mut node.pair.1)
            })
    }
}
