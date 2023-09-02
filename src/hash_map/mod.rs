use crate::allocator::DefaultAllocator;
use crate::equals::{EqualTo, Equals};
use crate::hash_map::entry::Entry;
use crate::{
    allocator::Allocator,
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};
use std::fmt::{Debug, Formatter};

use self::iter::{Iter, IterMut};

pub mod entry;
pub mod iter;

/// Hash map with the default allocator.
pub type DefaultHashMap<K, V, H = DefaultHash<K>, E = EqualTo<K>> =
    HashMap<K, V, DefaultAllocator, H, E>;

/// A hash map that can store and fetch values from a key in O(1) time
#[repr(C)]
pub struct HashMap<K: Eq, V, A: Allocator, H: Hash<K> = DefaultHash<K>, E: Equals<K> = EqualTo<K>> {
    hash_table: HashTable<K, V, A, H, E>,
}

impl<K: Eq, V, A: Allocator + Default> HashMap<K, V, A, DefaultHash<K>, EqualTo<K>>
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

impl<K: Eq, V, A: Allocator, H: Hash<K>, E: Equals<K>> HashMap<K, V, A, H, E> {
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

    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    ///
    /// `key`: The key.
    pub fn entry(&mut self, key: K) -> Entry<K, V, A, H, E> {
        self.hash_table.entry(key).into()
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

    /// Inserts the key-value pair into the hash map, returning the old value in the map
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

    /// Creates a hash map backed by an allocator
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
}

impl<K: Debug + Eq, V: Debug, A: Allocator, H: Hash<K>, E: Equals<K>> Debug
    for HashMap<K, V, A, H, E>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.iter()
                .map(|(k, v)| format!("{k:?}: {v:?}"))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl<K: Eq, V, A: Allocator + Default> Default for HashMap<K, V, A, DefaultHash<K>, EqualTo<K>>
where
    DefaultHash<K>: Hash<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq, V, A: Allocator + Default> FromIterator<(K, V)>
    for HashMap<K, V, A, DefaultHash<K>, EqualTo<K>>
where
    DefaultHash<K>: Hash<K>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            hash_table: HashTable::from_iter(iter),
        }
    }
}

unsafe impl<K: Eq + Send, V: Send, A: Allocator + Send, H: Hash<K>, E: Equals<K>> Send
    for HashMap<K, V, A, H, E>
{
}
unsafe impl<K: Eq + Sync, V: Sync, A: Allocator + Sync, H: Hash<K>, E: Equals<K>> Sync
    for HashMap<K, V, A, H, E>
{
}

#[cfg(test)]
mod test {
    use crate::hash_map::DefaultHashMap;
    use std::collections::BTreeMap;

    #[test]
    fn iter() {
        let reference_map: BTreeMap<u32, u32> =
            (0..10).map(|n| n * 10).map(|n| (n, n + 2)).collect();
        let hm: DefaultHashMap<u32, u32> = reference_map.iter().map(|(k, v)| (*k, *v)).collect();
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
        let mut hm: DefaultHashMap<u32, u32> =
            reference_map.iter().map(|(k, v)| (*k, *v)).collect();
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
