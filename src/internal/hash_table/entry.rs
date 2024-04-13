use crate::allocator::Allocator;
use crate::equals::Equals;
use crate::hash::Hash;
use crate::internal::hash_table::node::Node;
use crate::internal::hash_table::HashTable;

/// A vacant node - one that has not been inserted yet.
pub struct VacantEntry<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>> {
    pub(crate) table: &'a mut HashTable<K, V, A, H, E>,
    pub(crate) target_bucket: &'a mut *mut Node<K, V>,
    pub(crate) key: K,
}

/// An entry in a hash table.
pub enum Entry<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>> {
    /// There was a node found already for the key.
    Occupied(&'a mut Node<K, V>),
    /// There was not a node already present for the key.
    Vacant(VacantEntry<'a, K, V, A, H, E>),
}

impl<'a, K: PartialEq, V, A: Allocator, H: Hash<K>, E: Equals<K>> Entry<'a, K, V, A, H, E> {
    /// Provides in-place mutable access to the value.
    ///
    /// # Arguments
    ///
    /// `f`: A function taking a mutable reference to the value.
    pub fn and_modify<F: Fn(&mut V)>(mut self, f: F) -> Self {
        if let Self::Occupied(occupied) = &mut self {
            f(&mut occupied.val);
        }

        self
    }

    /// Fetches the value stored in the entry, or inserts a default key.
    ///
    /// # Arguments
    ///
    /// `default`: The default value.  
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    /// Fetches the value stored in the entry, or inserts a default key.
    ///
    /// # Arguments
    ///
    /// `default`: A function producing a default value.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(v) => &mut v.val,
            Self::Vacant(entry) => {
                let val = default();
                &mut entry
                    .table
                    .insert_impl(entry.target_bucket, entry.key, val)
                    .val
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::internal::hash_table::entry::Entry;
    use crate::internal::hash_table::DefaultHashTable;

    #[test]
    fn occupied() {
        let mut ht = DefaultHashTable::new();
        ht.insert("def", 5);

        assert!(matches!(ht.entry("def"), Entry::Occupied(_)));
    }

    #[test]
    fn occupied_and_modify() {
        let mut ht = DefaultHashTable::new();
        ht.insert("def", 5);

        assert_eq!(
            *ht.entry("def").and_modify(|val| *val *= 2).or_insert(6),
            10
        );
        assert_eq!(ht.get(&"def"), Some((&"def", &10)))
    }

    #[test]
    fn or_insert() {
        let mut ht = DefaultHashTable::new();

        assert_eq!(*ht.entry("abc").or_insert(5), 5);
    }

    #[test]
    fn or_insert_with() {
        let mut ht = DefaultHashTable::new();
        let mut counter = 0;

        assert_eq!(
            *ht.entry("abc").or_insert_with(|| {
                counter += 1;
                5
            }),
            5
        );
        assert_eq!(
            *ht.entry("abc").or_insert_with(|| {
                counter += 1;
                10
            }),
            5
        );
        assert_eq!(counter, 1);
    }

    #[test]
    fn vacant() {
        let mut ht = DefaultHashTable::new();
        ht.insert("def", 5);

        assert!(matches!(ht.entry("abc"), Entry::Vacant(_)));
    }
}
