use crate::internal::hash_table::iter::CompatIter;

/// An iterator that produces keys in a hash set
/// in an unspecified order. This is not binary
/// compatible with C++.
///
/// # Notes
///
/// It is unspecified whether or not an element
/// inserted after an iterator was created will
/// be yielded by the iterator
pub struct Iter<'a, K: Eq> {
    inner: crate::internal::hash_table::iter::Iter<'a, K, ()>,
}

impl<'a, K: Eq> Iter<'a, K> {
    /// Converts the Rust iterator into a pair of
    /// `(begin, end)` compatibility iterators
    pub fn into_compat(self) -> (CompatIter<K, ()>, CompatIter<K, ()>) {
        self.inner.into_compat()
    }

    /// Constructs a Rust iterator from a pair of
    /// compatibility iterators
    ///
    /// # Safety
    ///
    /// The compatibility iterators specified must
    /// point to valid portions of the hash table
    ///
    /// # Arguments
    ///
    /// `begin`: The starting compatibility iterator
    ///
    /// `end`: The ending compatibility iterator
    pub unsafe fn from_compat(begin: CompatIter<K, ()>, end: CompatIter<K, ()>) -> Self {
        Self {
            inner: crate::internal::hash_table::iter::Iter::from_compat(begin, end),
        }
    }

    /// Creates a new hash table iterator from the
    /// hash table's buckets
    ///
    /// # Arguments
    ///
    /// `buckets`: The slice of buckets owned by the
    /// hash table
    pub(crate) fn new(inner: crate::internal::hash_table::iter::Iter<'a, K, ()>) -> Self {
        Self { inner }
    }
}

impl<'a, K: Eq> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}
