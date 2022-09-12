use crate::{
    allocator::{Allocator, DefaultAllocator},
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};

use self::iter::Iter;

pub mod iter;

/// A hash set that can store and fetch keys in O(1) time
#[repr(C)]
pub struct HashSet<K: Eq, H: Hash<K> = DefaultHash<K>, A: Allocator = DefaultAllocator> {
    hash_table: HashTable<K, (), H, A>,
}

impl<K: Eq> HashSet<K, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    /// Creates a new empty hashset
    pub fn new() -> Self {
        Self {
            hash_table: HashTable::new(),
        }
    }
}

impl<K: Eq, H: Hash<K>, A: Allocator> HashSet<K, H, A> {
    /// Clears the hash set, removing all keys
    pub fn clear(&mut self) {
        self.hash_table.clear()
    }

    /// Checks if the hashset contains the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn contains_key(&self, key: &K) -> bool {
        self.hash_table.contains_key(key)
    }

    /// Fetches the key from the hashset
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get(&self, key: &K) -> Option<&K> {
        self.hash_table.get(key).map(|(k, _)| k)
    }

    /// Inserts the key pair into the hashset. Returns true on success
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K) -> bool {
        self.hash_table.insert(key, ()).is_some()
    }

    /// Creates a hash set backed by an allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator to use to allocate and de-allocate memory
    ///
    /// # Safety
    ///
    /// The allocator must safely allocate and de-allocate valid memory
    pub unsafe fn new_in(allocator: A) -> Self {
        Self {
            hash_table: HashTable::new_in(allocator),
        }
    }

    /// Returns true if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.hash_table.is_empty()
    }

    /// Returns an iterator over the hash set's keys
    pub fn iter(&self) -> Iter<K> {
        Iter::new(self.hash_table.iter())
    }

    /// Returns the number of elements in the hash set
    pub fn len(&self) -> usize {
        self.hash_table.len()
    }

    /// Removes a key from the hash set, returning it if it was found
    pub fn remove(&mut self, key: &K) -> Option<K> {
        self.hash_table.remove_entry(key).map(|(key, _)| key)
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
            hash_table: HashTable::from_iter(iter.into_iter().map(|k| (k, ()))),
        }
    }
}

unsafe impl<K: Eq + Send, H: Hash<K>, A: Allocator + Send> Send for HashSet<K, H, A> {}
unsafe impl<K: Eq + Sync, H: Hash<K>, A: Allocator + Sync> Sync for HashSet<K, H, A> {}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::HashSet;

    #[test]
    fn iter() {
        let reference_map: BTreeSet<u32> = (0..10).map(|n| n * 10).collect();
        let hm: HashSet<u32> = reference_map.iter().map(|k| *k).collect();
        assert_eq!(
            hm.iter().map(|k| *k).collect::<BTreeSet<u32>>(),
            reference_map
        );
    }
}
