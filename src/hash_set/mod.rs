use crate::allocator::DefaultAllocator;
use crate::equals::{EqualTo, Equals};
use crate::{
    allocator::Allocator,
    hash::{DefaultHash, Hash},
    internal::hash_table::HashTable,
};
use std::fmt::{Debug, Formatter};

use self::iter::Iter;

pub mod iter;

/// Hash set with the default allocator.
pub type DefaultHashSet<K, H = DefaultHash<K>, E = EqualTo<K>> = HashSet<K, DefaultAllocator, H, E>;

/// A hash set that can store and fetch keys in O(1) time
#[repr(C)]
pub struct HashSet<
    K: PartialEq,
    A: Allocator,
    H: Hash<K> = DefaultHash<K>,
    E: Equals<K> = EqualTo<K>,
> {
    hash_table: HashTable<K, (), A, H, E>,
}

impl<K: PartialEq, A: Allocator + Default> HashSet<K, A, DefaultHash<K>, EqualTo<K>>
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

impl<K: PartialEq, A: Allocator, H: Hash<K>, E: Equals<K>> HashSet<K, A, H, E> {
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

impl<K: Debug + PartialEq, A: Allocator, H: Hash<K>, E: Equals<K>> Debug for HashSet<K, A, H, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.iter()
                .map(|k| format!("{k:?}"))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl<K: PartialEq, A: Allocator + Default> Default for HashSet<K, A, DefaultHash<K>, EqualTo<K>>
where
    DefaultHash<K>: Hash<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, A: Allocator + Default> FromIterator<K>
    for HashSet<K, A, DefaultHash<K>, EqualTo<K>>
where
    DefaultHash<K>: Hash<K>,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        Self {
            hash_table: HashTable::from_iter(iter.into_iter().map(|k| (k, ()))),
        }
    }
}

unsafe impl<K: PartialEq + Send, A: Allocator + Send, H: Hash<K>, E: Equals<K>> Send
    for HashSet<K, A, H, E>
{
}
unsafe impl<K: PartialEq + Sync, A: Allocator + Sync, H: Hash<K>, E: Equals<K>> Sync
    for HashSet<K, A, H, E>
{
}

#[cfg(test)]
mod test {
    use crate::hash_set::DefaultHashSet;
    use std::collections::BTreeSet;

    #[test]
    fn iter() {
        let reference_map: BTreeSet<u32> = (0..10).map(|n| n * 10).collect();
        let hm: DefaultHashSet<u32> = reference_map.iter().copied().collect();
        assert_eq!(hm.iter().copied().collect::<BTreeSet<u32>>(), reference_map);
    }
}
