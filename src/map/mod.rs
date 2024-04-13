use crate::internal::rb_tree::iter::{Iter, IterMut};
use crate::{
    allocator::Allocator,
    compare::{Compare, Less},
    internal::rb_tree::RBTree,
};
use duplicate::duplicate_item;
use std::fmt::{Debug, Formatter};

/// A map backed by a red-black tree that is always ordered.
/// Insertion, lookup, and removal are O(nlgn). If you do not
/// need ordering, look at `HashMap`, which takes O(1) time
/// for those operations
#[derive(Default)]
#[repr(C)]
pub struct Map<K: PartialEq, V, A: Allocator, C: Compare<K> = Less<K>> {
    pub(crate) inner: RBTree<K, V, A, C>,
}

impl<K: PartialEq, V, A: Allocator, C: Compare<K> + Default> Map<K, V, A, C> {
    /// Constructs a map using a specified allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            inner: RBTree::with_allocator(allocator),
        }
    }
}

impl<K: PartialEq, V, A: Allocator + Default, C: Compare<K>> Map<K, V, A, C> {
    /// Constructs a map using a specified comparator
    ///
    /// # Arguments
    ///
    /// `compare`: The comparator
    pub fn with_compare(compare: C) -> Self {
        Self {
            inner: RBTree::with_compare(compare),
        }
    }
}

impl<K: PartialEq, V, A: Allocator, C: Compare<K>> Map<K, V, A, C> {
    /// Constructs a map using a specified allocator
    /// and comparator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    ///
    /// `compare`: The comparator
    pub fn with_allocator_and_compare(allocator: A, compare: C) -> Self {
        Self {
            inner: RBTree::with_allocator_and_compare(allocator, compare),
        }
    }

    /// Clears the map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns true if the map contains a pair indexed
    /// by the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Fetches the value indexed by the key in the map
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Fetches the value indexed by the key in the map
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Inserts a key-value pair into the map
    ///
    /// # Arguments
    ///
    /// `key`: The key to insert and index by
    ///
    /// `value`: The value to insert
    fn _insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner._insert(key, value)
    }

    /// Returns true if the map contains no elements
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over the elements in the tree.
    ///
    /// # Safety
    /// This iterator is not tested as trees are only partially implemented.
    #[duplicate_item(
        iter        Self        Iter;
        [iter]      [&Self]     [Iter];
        [iter_mut]  [&mut Self] [IterMut];
    )]
    #[allow(clippy::needless_arbitrary_self_type)]
    pub unsafe fn iter(self: Self) -> Iter<K, V> {
        self.inner.iter()
    }

    /// Returns the number of elements in the map
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Removes a key-value pair from the map,
    /// returning the element if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair

    fn _remove(&mut self, key: &K) -> Option<V> {
        self.inner._remove(key)
    }

    /// Removes a key-value pair from the map,
    /// returning the pair if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        self.inner.remove_entry(key)
    }
}

impl<K: PartialEq + Debug, V: Debug, A: Allocator, C: Compare<K>> Debug for Map<K, V, A, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            unsafe { self.iter() }
                .map(|(k, v)| format!("{k:?}: {v:?}"))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

unsafe impl<K: PartialEq + Send, V: Send, A: Allocator + Send, C: Compare<K> + Send> Send
    for Map<K, V, A, C>
{
}

unsafe impl<K: PartialEq + Sync, V: Sync, A: Allocator + Sync, C: Compare<K> + Sync> Sync
    for Map<K, V, A, C>
{
}
