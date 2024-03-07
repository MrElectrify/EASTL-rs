use crate::internal::rb_tree::node::Node;
use std::marker::PhantomData;

/// An iterator over a Red-Black tree's nodes.
pub struct Iter<'a, K, V> {
    pub(super) node: *mut Node<K, V>,
    pub(super) _marker: PhantomData<&'a ()>,
}

pub struct IterMut<'a, K, V> {
    pub(super) node: *mut Node<K, V>,
    pub(super) _marker: PhantomData<&'a mut ()>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.node.as_ref() }
            .and_then(Node::next)
            .map(|node| (node.key(), node.val()))
    }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.node.as_mut() }
            .and_then(Node::next_mut)
            .map(|node| (&node.pair.0, &mut node.pair.1))
    }
}
