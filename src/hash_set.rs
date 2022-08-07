use crate::{
    allocator::{Allocator, DefaultAllocator},
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};

/// A hash set that can store and fetch keys in O(1) time
#[repr(C)]
pub struct HashSet<K: Eq, H: Hash<K>, A: Allocator> {
    hashtable: HashTable<K, (), H, A>,
}

impl<K: Eq> HashSet<K, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    /// Creates a new empty hashset
    pub fn new() -> Self {
        Self {
            hashtable: HashTable::new(),
        }
    }
}

impl<K: Eq, H: Hash<K>, A: Allocator> HashSet<K, H, A> {
    /// Clears the hash set, removing all keys
    pub fn clear(&mut self) {
        self.hashtable.clear()
    }

    /// Checks if the hashset contains the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn contains_key(&self, key: &K) -> bool {
        self.hashtable.contains_key(key)
    }

    /// Fetches the key from the hashset
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get(&self, key: &K) -> Option<&K> {
        self.hashtable.get(key).map(|(k, _)| k)
    }

    /// Inserts the key pair into the hashset. Returns true on success
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K) -> bool {
        self.hashtable.insert(key, ()).is_some()
    }

    /// Returns true if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.hashtable.is_empty()
    }

    /// Returns the number of elements in the hash set
    pub fn len(&self) -> usize {
        self.hashtable.len()
    }

    /// Removes a key from the hash set, returning it if it was found
    pub fn remove(&mut self, key: &K) -> Option<K> {
        self.hashtable.remove_entry(key).map(|(key, _)| key)
    }

    /// Creates a hash set backed by an allocator
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            hashtable: HashTable::with_allocator(allocator),
        }
    }
}

impl<K: Eq> Default for HashSet<K, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq> FromIterator<K> for HashSet<K, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        Self {
            hashtable: HashTable::from_iter(iter.into_iter().map(|k| (k, ()))),
        }
    }
}
