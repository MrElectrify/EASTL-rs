use crate::allocator::Allocator;
use crate::equals::Equals;
use crate::hash::Hash;
use crate::internal::hash_table;

/// An entry in a hash map.
pub struct Entry<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>>(
    hash_table::entry::Entry<'a, K, V, A, H, E>,
);

impl<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>> Entry<'a, K, V, A, H, E> {
    /// Provides in-place mutable access to the value.
    ///
    /// # Arguments
    ///
    /// `f`: A function taking a mutable reference to the value.
    pub fn and_modify<F: Fn(&mut V)>(self, f: F) -> Self {
        self.0.and_modify(f).into()
    }

    /// Fetches the value stored in the entry, or inserts a default key.
    ///
    /// # Arguments
    ///
    /// `default`: The default value.  
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.0.or_insert(default)
    }

    /// Fetches the value stored in the entry, or inserts a default key.
    ///
    /// # Arguments
    ///
    /// `default`: A function producing a default value.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with(default)
    }
}

impl<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>>
    From<hash_table::entry::Entry<'a, K, V, A, H, E>> for Entry<'a, K, V, A, H, E>
{
    fn from(value: hash_table::entry::Entry<'a, K, V, A, H, E>) -> Self {
        Self(value)
    }
}
