use crate::{
    allocator::{Allocator, DefaultAllocator},
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};

/// A hash map that can store and fetch values from a key in O(1) time
#[repr(C)]
pub struct HashMap<K: Eq, V, H: Hash<K>, A: Allocator> {
    hashtable: HashTable<K, V, H, A>,
}

impl<K: Eq, V> HashMap<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    /// Creates a new empty hash map
    pub fn new() -> Self {
        Self {
            hashtable: HashTable::new(),
        }
    }
}

impl<K: Eq, V, H: Hash<K>, A: Allocator> HashMap<K, V, H, A> {
    /// Clears the hash map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.hashtable.clear()
    }

    /// Checks if the hash map contains the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn contains_key(&self, key: &K) -> bool {
        self.hashtable.contains_key(key)
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get(&self, key: &K) -> Option<&V> {
        self.hashtable.get(key).map(|(_, v)| v)
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.hashtable.get_mut(key)
    }

    /// Inserts the key-value pair into the hash map
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.hashtable.insert(key, value)
    }

    /// Returns true if the hash map is empty
    pub fn is_empty(&self) -> bool {
        self.hashtable.is_empty()
    }

    /// Returns the number of key-value pairs in the hash map
    pub fn len(&self) -> usize {
        self.hashtable.len()
    }

    /// Removes a key-value pair from the hash map,
    /// returning the element if it was found
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.hashtable.remove(key)
    }

    /// Removes a key-value pair from the hash map,
    /// returning the pair if it was found
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        self.hashtable.remove_entry(key)
    }

    /// Creates a hash map backed by an allocator
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            hashtable: HashTable::with_allocator(allocator),
        }
    }
}

impl<K: Eq, V> Default for HashMap<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq, V> FromIterator<(K, V)> for HashMap<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            hashtable: HashTable::from_iter(iter),
        }
    }
}
