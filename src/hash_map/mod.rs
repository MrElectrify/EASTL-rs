use crate::{
    allocator::{Allocator, DefaultAllocator},
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};

use self::iter::{Iter, IterMut};

pub mod iter;

/// A hash map that can store and fetch values from a key in O(1) time
#[repr(C)]
pub struct HashMap<K: Eq, V, H: Hash<K> = DefaultHash<K>, A: Allocator = DefaultAllocator> {
    hash_table: HashTable<K, V, H, A>,
}

impl<K: Eq, V> HashMap<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    /// Creates a new empty hash map
    pub fn new() -> Self {
        Self {
            hash_table: HashTable::new(),
        }
    }
}

impl<K: Eq, V, H: Hash<K>, A: Allocator> HashMap<K, V, H, A> {
    /// Clears the hash map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.hash_table.clear()
    }

    /// Checks if the hash map contains the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn contains_key(&self, key: &K) -> bool {
        self.hash_table.contains_key(key)
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get(&self, key: &K) -> Option<&V> {
        self.hash_table.get(key).map(|(_, v)| v)
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.hash_table.get_mut(key)
    }

    /// Inserts the key-value pair into the hash map
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.hash_table.insert(key, value)
    }

    /// Returns true if the hash map is empty
    pub fn is_empty(&self) -> bool {
        self.hash_table.is_empty()
    }

    /// Returns an iterator over the hash map's
    /// key-value pairs
    pub fn iter(&self) -> Iter<K, V> {
        self.hash_table.iter()
    }

    /// Returns an iterator over the hash map's
    /// key-value pairs, where the values are
    /// mutable
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.hash_table.iter_mut()
    }

    /// Returns the number of key-value pairs in the hash map
    pub fn len(&self) -> usize {
        self.hash_table.len()
    }

    /// Removes a key-value pair from the hash map,
    /// returning the element if it was found
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.hash_table.remove(key)
    }

    /// Removes a key-value pair from the hash map,
    /// returning the pair if it was found
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        self.hash_table.remove_entry(key)
    }

    /// Creates a hash map backed by an allocator
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            hash_table: HashTable::with_allocator(allocator),
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
            hash_table: HashTable::from_iter(iter),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::HashMap;

    #[test]
    fn iter() {
        let reference_map: BTreeMap<u32, u32> =
            (0..10).map(|n| n * 10).map(|n| (n, n + 2)).collect();
        let hm: HashMap<u32, u32> = reference_map.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(
            hm.iter()
                .map(|(k, v)| (*k, *v))
                .collect::<BTreeMap<u32, u32>>(),
            reference_map
        );
    }

    #[test]
    fn iter_mut() {
        let mut reference_map: BTreeMap<u32, u32> =
            (0..10).map(|n| n * 10).map(|n| (n, n + 2)).collect();
        let mut hm: HashMap<u32, u32> = reference_map.iter().map(|(k, v)| (*k, *v)).collect();
        reference_map.iter_mut().for_each(|(_, v)| *v *= 2);
        hm.iter_mut().for_each(|(_, v)| *v *= 2);
        assert_eq!(
            hm.iter()
                .map(|(k, v)| (*k, *v))
                .collect::<BTreeMap<u32, u32>>(),
            reference_map
        );
    }
}
