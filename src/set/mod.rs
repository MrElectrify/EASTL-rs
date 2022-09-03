use crate::{
    allocator::{Allocator, DefaultAllocator},
    compare::{Compare, Less},
    internal::rb_tree::RBTree,
};

/// A set backed by a red-black tree that is always ordered.
/// Insertion, lookup, and removal are O(nlgn). If you do not
/// need ordering, look at `HashSet`, which takes O(1) time
/// for those operations
#[derive(Default)]
pub struct Set<K: Eq, C: Compare<K> = Less<K>, A: Allocator = DefaultAllocator> {
    inner: RBTree<K, (), C, A>,
}

impl<K: Eq, C: Compare<K> + Default, A: Allocator> Set<K, C, A> {
    /// Constructs a set using a specified allocator
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

impl<K: Eq, C: Compare<K>, A: Allocator + Default> Set<K, C, A> {
    /// Constructs a set using a specified comparator
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

impl<K: Eq, C: Compare<K>, A: Allocator> Set<K, C, A> {
    /// Constructs a set using a specified allocator
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

    /// Clears the set, removing all key-value pairs
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns true if the map contains a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to index
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Inserts a key into the set. Returns true on success
    ///
    /// # Arguments
    ///
    /// `key`: The key to insert and index by
    fn _insert(&mut self, key: K) -> bool {
        self.inner._insert(key, ()).is_some()
    }

    /// Returns true if the set contains no elements
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of elements in the set
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Removes a key from the set,
    /// returning the element if it was found
    ///
    fn _remove(&mut self, key: &K) -> Option<K> {
        self.inner.remove_entry(key).map(|(k, _)| k)
    }
}
