/// An iterator that produces key-value pairs
/// in a hash map in an unspecified order. This
/// is not binary compatible with C++.
///
/// # Notes
///
/// It is unspecified whether or not an element
/// inserted after an iterator was created will
/// be yielded by the iterator
pub type Iter<'a, K, V> = crate::internal::hash_table::iter::Iter<'a, K, V>;

/// An iterator that produces key-value pairs
/// in a hash map in an unspecified order. The
/// values yielded by the iterator are mutable. This
/// is not binary compatible with C++, but can
/// be constructed with a pair of `CompatIter`
/// in the case that binary-compatible iterators
/// are required
///
/// # Notes
///
/// It is unspecified whether or not an element
/// inserted after an iterator was created will
/// be yielded by the iterator
pub type IterMut<'a, K, V> = crate::internal::hash_table::iter::IterMut<'a, K, V>;
