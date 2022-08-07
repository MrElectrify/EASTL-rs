use super::node::Node;

/// A compatibility iterator for C++ iterators.
/// Tho concept of begin and end are not used in
/// rust, so these are strictly for compatibility's
/// sake. They cannot actually be used to iterate.
#[repr(C)]
pub struct CompatIter<K: Eq, V> {
    bucket_ptr: *const *const Node<K, V>,
    node_ptr: *mut Node<K, V>,
}

/// A compatibility mutable iterator for C++ iterators.
/// Tho concept of begin and end are not used in
/// rust, so these are strictly for compatibility's
/// sake. They cannot actually be used to iterate.
#[repr(C)]
pub struct CompatIterMut<K: Eq, V> {
    bucket_ptr: *const *mut Node<K, V>,
    node_ptr: *mut Node<K, V>,
}

/// An iterator that produces key-value pairs
/// in a hash table in an unspecified order. This
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
#[derive(Clone)]
struct RawIter<'a, K: Eq, V> {
    bucket_iter: std::slice::Iter<'a, *mut Node<K, V>>,
    node_ptr: *mut Node<K, V>,
}

impl<'a, K: Eq, V> RawIter<'a, K, V> {
    /// Converts the Rust iterator into a pair of
    /// `(begin, end)` compatibility iterators
    fn into_compat(self) -> (CompatIter<K, V>, CompatIter<K, V>) {
        (
            CompatIter::<K, V> {
                bucket_ptr: self.bucket_iter.as_slice().as_ptr() as *const *const Node<K, V>,
                node_ptr: self.node_ptr,
            },
            CompatIter::<K, V> {
                bucket_ptr: unsafe {
                    (self.bucket_iter.as_slice().as_ptr() as *const *const Node<K, V>)
                        .add(self.bucket_iter.as_slice().len())
                },
                node_ptr: std::ptr::null_mut(),
            },
        )
    }

    /// Converts the Rust iterator into a pair of
    /// mutable `(begin, end)` compatibility iterators
    ///
    /// # Safety
    ///
    /// Mutability of the exposed iterator must be enforced.
    /// POLLO: Is there a better way to do this?
    unsafe fn into_compat_mut(self) -> (CompatIterMut<K, V>, CompatIterMut<K, V>) {
        (
            CompatIterMut::<K, V> {
                bucket_ptr: self.bucket_iter.as_slice().as_ptr(),
                node_ptr: self.node_ptr,
            },
            CompatIterMut::<K, V> {
                bucket_ptr: self
                    .bucket_iter
                    .as_slice()
                    .as_ptr()
                    .add(self.bucket_iter.as_slice().len()),
                node_ptr: std::ptr::null_mut(),
            },
        )
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
    unsafe fn from_compat(begin: CompatIter<K, V>, end: CompatIter<K, V>) -> Self {
        Self {
            bucket_iter: std::slice::from_raw_parts(
                begin.bucket_ptr as *const *mut Node<K, V>,
                end.bucket_ptr.offset_from(begin.bucket_ptr) as usize,
            )
            .iter(),
            node_ptr: begin.node_ptr,
        }
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
    unsafe fn from_compat_mut(begin: CompatIterMut<K, V>, end: CompatIterMut<K, V>) -> Self {
        Self {
            bucket_iter: std::slice::from_raw_parts(
                begin.bucket_ptr,
                end.bucket_ptr.offset_from(begin.bucket_ptr) as usize,
            )
            .iter(),
            node_ptr: begin.node_ptr,
        }
    }

    /// Creates a new hash table iterator from the
    /// hash table's buckets
    ///
    /// # Arguments
    ///
    /// `buckets`: The slice of buckets owned by the
    /// hash table
    fn new(buckets: &'a [*mut Node<K, V>]) -> Self {
        let mut new_iter = Self {
            bucket_iter: buckets.iter(),
            node_ptr: std::ptr::null_mut(),
        };
        // find the first next node
        new_iter.node_ptr = new_iter.next_bucket().unwrap_or_else(std::ptr::null_mut);
        new_iter
    }

    /// Finds the next non-null bucket
    fn next_bucket(&mut self) -> Option<*mut Node<K, V>> {
        for &ptr in self.bucket_iter.by_ref() {
            if !ptr.is_null() {
                return Some(ptr);
            }
        }
        None
    }
}

impl<'a, K: Eq, V> Iterator for RawIter<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        // the only way for the node pointer to be null is
        // if we already hit the end
        if self.node_ptr.is_null() {
            return None;
        }
        // traverse to the next node, but return the current
        let old_ptr = self.node_ptr;
        self.node_ptr = unsafe { Some((*self.node_ptr).next) }
            .filter(|next_node| !next_node.is_null())
            .or_else(|| self.next_bucket())
            .unwrap_or_else(std::ptr::null_mut);
        // of course it is safe to deref old_ptr here because
        // we have already verified it is non-null
        Some(unsafe { (&(*old_ptr).key, &mut (*old_ptr).val) })
    }
}

/// An iterator that produces key-value pairs
/// in a hash table in an unspecified order. This
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
#[derive(Clone)]
pub struct Iter<'a, K: Eq, V> {
    inner: RawIter<'a, K, V>,
}

impl<'a, K: Eq, V> Iter<'a, K, V> {
    /// Converts the Rust iterator into a pair of
    /// `(begin, end)` compatibility iterators
    pub fn into_compat(self) -> (CompatIter<K, V>, CompatIter<K, V>) {
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
    pub unsafe fn from_compat(begin: CompatIter<K, V>, end: CompatIter<K, V>) -> Self {
        Self {
            inner: RawIter::from_compat(begin, end),
        }
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
    pub unsafe fn from_compat_mut(begin: CompatIterMut<K, V>, end: CompatIterMut<K, V>) -> Self {
        Self {
            inner: RawIter::from_compat_mut(begin, end),
        }
    }

    /// Creates a new hash table iterator from the
    /// hash table's buckets
    ///
    /// # Arguments
    ///
    /// `buckets`: The slice of buckets owned by the
    /// hash table
    pub(crate) fn new(buckets: &'a [*mut Node<K, V>]) -> Self {
        Self {
            inner: RawIter::new(buckets),
        }
    }
}

impl<'a, K: Eq, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k, &*v))
    }
}

/// An iterator that produces key-value pairs
/// in a hash table in an unspecified order. The
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
#[derive(Clone)]
pub struct IterMut<'a, K: Eq, V> {
    inner: RawIter<'a, K, V>,
}

impl<'a, K: Eq, V> IterMut<'a, K, V> {
    /// Converts the Rust iterator into a pair of
    /// `(begin, end)` compatibility iterators
    pub fn into_compat(self) -> (CompatIter<K, V>, CompatIter<K, V>) {
        self.inner.into_compat()
    }

    /// Converts the Rust iterator into a pair of
    /// mutable `(begin, end)` compatibility iterators
    pub fn into_compat_mut(self) -> (CompatIterMut<K, V>, CompatIterMut<K, V>) {
        unsafe { self.inner.into_compat_mut() }
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
    pub unsafe fn from_compat(begin: CompatIter<K, V>, end: CompatIter<K, V>) -> Self {
        Self {
            inner: RawIter::from_compat(begin, end),
        }
    }

    /// Creates a new hash table iterator from the
    /// hash table's buckets
    ///
    /// # Arguments
    ///
    /// `buckets`: The slice of buckets owned by the
    /// hash table
    pub(crate) fn new(buckets: &'a [*mut Node<K, V>]) -> Self {
        Self {
            inner: RawIter::new(buckets),
        }
    }
}

impl<'a, K: Eq, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod test {
    use crate::internal::hash_table::HashTable;

    use super::RawIter;

    #[test]
    fn empty_iter() {
        let ht = HashTable::<u32, u32>::new();
        let mut iter = RawIter::new(unsafe {
            std::slice::from_raw_parts(ht.bucket_array, ht.bucket_count as usize)
        });
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter() {
        let ht = (0..10)
            .into_iter()
            .map(|n| (n, n))
            .collect::<HashTable<u32, u32>>();
        let mut iter = RawIter::new(unsafe {
            std::slice::from_raw_parts(ht.bucket_array, ht.bucket_count as usize)
        });
        for _ in 0..10 {
            assert!(iter.next().is_some());
        }
        // should be empty now
        assert!(iter.next().is_none());
    }

    #[test]
    fn cloned() {
        let ht = (0..10)
            .into_iter()
            .map(|n| n * 11)
            .map(|n| (n, n))
            .collect::<HashTable<u32, u32>>();
        let mut iter = RawIter::new(unsafe {
            std::slice::from_raw_parts(ht.bucket_array, ht.bucket_count as usize)
        });
        for _ in 0..5 {
            assert!(iter.next().is_some());
        }
        // clone the iter
        let mut cloned = iter.clone();
        for _ in 0..5 {
            assert!(iter.next().is_some());
            assert!(cloned.next().is_some());
        }
        assert!(iter.next().is_none());
        assert!(cloned.next().is_none())
    }

    #[test]
    fn compat_sanity() {
        let ht = (0..10)
            .into_iter()
            .map(|n| n * 10)
            .map(|n| (n, n))
            .collect::<HashTable<u32, u32>>();
        let iter = RawIter::new(unsafe {
            std::slice::from_raw_parts(ht.bucket_array, ht.bucket_count as usize)
        });
        let (begin, end) = iter.into_compat();
        let mut iter = unsafe { RawIter::from_compat(begin, end) };
        for _ in 0..10 {
            assert!(iter.next().is_some());
        }
        // should be empty now
        assert!(iter.next().is_none());
    }

    #[test]
    fn compat_partial_use_sanity() {
        let ht = (0..10)
            .into_iter()
            .map(|n| n * 10)
            .map(|n| (n, n))
            .collect::<HashTable<u32, u32>>();
        let mut iter = RawIter::new(unsafe {
            std::slice::from_raw_parts(ht.bucket_array, ht.bucket_count as usize)
        });
        for _ in 0..5 {
            assert!(iter.next().is_some());
        }
        let (begin, end) = iter.into_compat();
        let mut iter = unsafe { RawIter::from_compat(begin, end) };
        for _ in 0..5 {
            assert!(iter.next().is_some());
        }
        // should be empty now
        assert!(iter.next().is_none());
    }
}
