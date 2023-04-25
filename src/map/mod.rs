use crate::{
    allocator::{Allocator, DefaultAllocator},
    compare::{Compare, Less},
    internal::rb_tree::RBTree,
};

/// A map backed by a red-black tree that is always ordered.
/// Insertion, lookup, and removal are O(nlgn). If you do not
/// need ordering, look at `HashMap`, which takes O(1) time
/// for those operations
#[derive(Default)]
pub struct Map<K: Eq, V, C: Compare<K> = Less<K>, A: Allocator = DefaultAllocator> {
    inner: RBTree<K, V, C, A>,
}

impl<K: Eq, V, C: Compare<K> + Default, A: Allocator> Map<K, V, C, A> {
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

impl<K: Eq, V, C: Compare<K>, A: Allocator + Default> Map<K, V, C, A> {
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

impl<K: Eq, V, C: Compare<K>, A: Allocator> Map<K, V, C, A> {
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

unsafe impl<K: Eq + Send, V: Send, C: Compare<K> + Send, A: Allocator + Send> Send
    for Map<K, V, C, A>
{
}

unsafe impl<K: Eq + Sync, V: Sync, C: Compare<K> + Sync, A: Allocator + Sync> Sync
    for Map<K, V, C, A>
{
}
