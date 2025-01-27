use crate::allocator::Allocator;
use crate::compare::Compare;
use crate::vector_map::VectorMap;

/// An occupied entry in a vector map.
pub struct OccupiedEntry<'a, K, V>(pub(super) &'a mut (K, V));

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Returns the value in the entry.
    pub fn get(&self) -> &V {
        &self.0 .1
    }

    /// Returns the value in the entry.
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.0 .1
    }

    /// Returns the key in the entry.
    pub fn key(&self) -> &K {
        &self.0 .0
    }
}

/// A vacant entry in a vector map.
pub struct VacantEntry<'a, K: PartialEq, V, A: Allocator, C: Compare<K>> {
    pub(super) vec: &'a mut VectorMap<K, V, A, C>,
    pub(super) key: K,
    pub(super) lower_bound: usize,
}

impl<'a, K: PartialEq, V, A: Allocator, C: Compare<K>> VacantEntry<'a, K, V, A, C> {
    /// Consumes the vacant entry and returns the key that would have been used.
    pub fn into_key(self) -> K {
        self.key
    }

    /// Inserts `value` at the key specified by the entry.
    pub fn insert(self, value: V) -> &'a mut V {
        self.vec.base.insert(self.lower_bound, (self.key, value));

        &mut self.vec.base[self.lower_bound].1
    }

    /// Returns the key that would be used to insert the entry.
    pub fn key(&self) -> &K {
        &self.key
    }
}

/// An entry in a vector map.
pub enum Entry<'a, K: PartialEq, V, A: Allocator, C: Compare<K>> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V, A, C>),
}
