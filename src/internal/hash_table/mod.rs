use std::marker::PhantomData;

use crate::{
    allocator::{Allocator, DefaultAllocator},
    hash::{DefaultHash, Hash},
};

use self::{
    iter::{Iter, IterMut},
    node::Node,
    rehash_policy::PrimeRehashPolicy,
};

pub mod iter;
pub mod node;
mod rehash_policy;

/// A base hashtable used to support hash maps and sets
#[repr(C)]
pub struct HashTable<K: Eq, V, H: Hash<K> = DefaultHash<K>, A: Allocator = DefaultAllocator> {
    /// The C++ object has some key extractor functor here
    /// that we don't need
    _pad: u8,
    bucket_array: *mut *mut Node<K, V>,
    bucket_count: u32,
    element_count: u32,
    rehash_policy: PrimeRehashPolicy,
    allocator: A,
    _phantom_key: PhantomData<K>,
    _phantom_value: PhantomData<V>,
    _phantom_hash: PhantomData<H>,
}

static EMPTY_BUCKET_ARR: [usize; 2] = [0; 2];

impl<K: Eq, V> HashTable<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    /// Creates an empty hashtable
    pub fn new() -> Self {
        unsafe { Self::new_in(DefaultAllocator::default()) }
    }
}

impl<K: Eq, V, H: Hash<K>, A: Allocator> HashTable<K, V, H, A> {
    /// Clears the hash table, removing all key-value pairs
    pub fn clear(&mut self) {
        self.free_buckets();
        self.element_count = 0;
    }

    /// Checks if the hash table contains the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get(&self, key: &K) -> Option<(&K, &V)> {
        let bucket = unsafe { (*self.bucket_for_key(key)).as_ref() };
        Self::find_in_bucket(bucket, key).map(|node| (node.key(), node.value()))
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let bucket = unsafe { (*self.bucket_for_key_mut(key)).as_mut() };
        Self::find_in_bucket_mut(bucket, key).map(|node| node.value_mut())
    }

    /// Inserts the key-value pair into the hash table
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut target_bucket = self.bucket_for_key_mut(&key);
        if let Some(existing_node) =
            Self::find_in_bucket_mut(unsafe { (*target_bucket).as_mut() }, &key)
        {
            Some(std::mem::replace(existing_node.value_mut(), value))
        } else {
            // check if we need to re-hash
            if let Some(bucket_count) =
                self.rehash_policy
                    .get_rehash_required(self.bucket_count, self.element_count, 1)
            {
                self.rehash(bucket_count);
                // update the target bucket
                target_bucket = self.bucket_for_key_mut(&key);
            }
            // allocate a new node and add it to the bucket
            let node = unsafe { self.allocator.allocate::<Node<K, V>>(1) };
            unsafe {
                std::ptr::write(
                    node as *mut Node<K, V>,
                    Node::<K, V>::new(key, value, target_bucket.read()),
                );
                target_bucket.write(node);
            };
            self.element_count += 1;
            None
        }
    }

    /// Returns true if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the hash table's
    /// key-value pairs
    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(self.buckets_imut())
    }

    /// Returns an iterator over the hash table's
    /// key-value pairs, where the values are
    /// mutable
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut::new(self.buckets_imut())
    }

    /// Returns the number of elements in the hash table
    pub fn len(&self) -> usize {
        self.element_count as usize
    }

    /// Creates a hash table backed by an allocator
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
            _pad: 0,
            bucket_array: unsafe { std::mem::transmute(EMPTY_BUCKET_ARR.as_ptr()) },
            bucket_count: 1,
            element_count: 0,
            rehash_policy: PrimeRehashPolicy::default(),
            allocator,
            _phantom_key: PhantomData::default(),
            _phantom_value: PhantomData::default(),
            _phantom_hash: PhantomData::default(),
        }
    }

    /// Removes a key-value pair from the hash table,
    /// returning the element if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, val)| val)
    }

    /// Removes a key-value pair from the hash table,
    /// returning the pair if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        // we need to trail behind by one so we can
        // update the correct pointer
        let mut bucket = self.bucket_for_key_mut(key);
        unsafe {
            while !(*bucket).is_null() && !(**bucket).matches(key) {
                bucket = &mut (**bucket).next;
            }
            if (*bucket).is_null() {
                None
            } else {
                let node = *bucket;
                (*bucket) = (**bucket).next;
                let key = std::ptr::read(&(*node).key);
                let value = std::ptr::read(&(*node).val);
                // notice we don't drop the key or value here.
                // we don't want to drop them now and still have
                // binary copies of them existing
                self.allocator.deallocate(node, 1);
                self.element_count -= 1;
                Some((key, value))
            }
        }
    }

    /// Fetches the bucket for a given key
    ///
    /// # Arguments
    ///
    /// `key`: The key
    fn bucket_for_key(&self, key: &K) -> *const *const Node<K, V> {
        &self.buckets()[Self::bucket_index(self.bucket_count, key)]
    }

    /// Fetches the bucket for a given key
    ///
    /// # Arguments
    ///
    /// `key`: The key
    fn bucket_for_key_mut(&mut self, key: &K) -> *mut *mut Node<K, V> {
        unsafe {
            self.bucket_array
                .add(Self::bucket_index(self.bucket_count, key))
        }
    }

    /// Returns the index of the bucket for the given
    /// hash key
    ///
    /// # Arguments
    ///
    /// `bucket_count`: The total number of buckets
    ///
    /// `key`: The key
    fn bucket_index(bucket_count: u32, key: &K) -> usize {
        let key_hash = H::hash(key);
        key_hash % bucket_count as usize
    }

    /// Returns the buckets for the hash table
    fn buckets(&self) -> &[*const Node<K, V>] {
        unsafe {
            std::slice::from_raw_parts(
                self.bucket_array as *const *const Node<K, V>,
                self.bucket_count as usize,
            )
        }
    }

    /// Returns the buckets for the hash table
    fn buckets_imut(&self) -> &[*mut Node<K, V>] {
        unsafe { std::slice::from_raw_parts(self.bucket_array, self.bucket_count as usize) }
    }

    /// Returns the buckets for the hash table
    fn buckets_mut(&mut self) -> &mut [*mut Node<K, V>] {
        unsafe { std::slice::from_raw_parts_mut(self.bucket_array, self.bucket_count as usize) }
    }

    /// Finds a key's node in a bucket
    ///
    /// # Arguments
    ///
    /// `bucket`: The bucket to search in
    fn find_in_bucket<'a>(mut bucket: Option<&'a Node<K, V>>, key: &K) -> Option<&'a Node<K, V>> {
        while let Some(node) = bucket {
            if node.matches(key) {
                return Some(node);
            }
            bucket = node.next();
        }
        None
    }

    /// Finds a key's node in a bucket
    ///
    /// # Arguments
    ///
    /// `bucket`:
    fn find_in_bucket_mut<'a>(
        mut bucket: Option<&'a mut Node<K, V>>,
        key: &K,
    ) -> Option<&'a mut Node<K, V>> {
        while let Some(node) = bucket {
            if node.matches(key) {
                return Some(node);
            }
            bucket = node.next_mut();
        }
        None
    }

    /// Frees a bucket and all of the child nodes
    ///
    /// # Arguments
    ///
    /// `bucket_node`: The node in the bucket
    fn free_bucket(&self, bucket_node: &mut Node<K, V>) {
        // free the next node
        if let Some(next_node) = bucket_node.next_mut() {
            self.free_bucket(next_node);
        }
        // drop and free our node
        unsafe {
            std::ptr::drop_in_place(bucket_node as *mut Node<K, V>);
            self.allocator.deallocate(bucket_node, 1)
        }
    }

    /// Frees all buckets
    fn free_buckets(&mut self) {
        if self.bucket_count > 1 {
            // we can't use `buckets_mut` here because it would cause us to
            // hold a mutable reference to self and later immutable. any ideas?
            let buckets = unsafe {
                std::slice::from_raw_parts_mut(self.bucket_array, self.bucket_count as usize)
            };
            for bucket in buckets.iter().filter_map(|elem| unsafe { elem.as_mut() }) {
                self.free_bucket(bucket)
            }
            // zero the pointers
            buckets.fill_with(std::ptr::null_mut)
        }
    }

    /// Rehash the table with a new bucket count
    ///
    /// # Arguments
    ///
    /// `bucket_count`: The desired bucket count
    fn rehash(&mut self, bucket_count: u32) {
        let new_buckets = unsafe {
            std::slice::from_raw_parts_mut(
                self.allocator.allocate(bucket_count as usize),
                bucket_count as usize,
            )
        };
        new_buckets.fill_with(std::ptr::null_mut);
        // transfer nodes over
        self.buckets_mut()
            .iter_mut()
            .filter(|bucket| !bucket.is_null())
            .for_each(|bucket_node_ref| {
                let mut bucket_node = *bucket_node_ref;
                while let Some(node) = unsafe { bucket_node.as_mut() } {
                    let new_index = Self::bucket_index(bucket_count, node.key());
                    let next_node = node.next;
                    node.next = new_buckets[new_index];
                    new_buckets[new_index] = node as *mut Node<K, V>;
                    bucket_node = next_node;
                }
                *bucket_node_ref = std::ptr::null_mut();
            });
        // free the old buckets before setting new ones
        self.free_buckets();
        self.bucket_array = new_buckets.as_mut_ptr();
        self.bucket_count = bucket_count;
    }
}

impl<K: Eq, V> Default for HashTable<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq, V, H: Hash<K>, A: Allocator> Drop for HashTable<K, V, H, A> {
    fn drop(&mut self) {
        self.free_buckets();
    }
}

impl<K: Eq, V> FromIterator<(K, V)> for HashTable<K, V, DefaultHash<K>, DefaultAllocator>
where
    DefaultHash<K>: Hash<K>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut ht = Self::default();
        iter.into_iter().for_each(|(k, v)| {
            ht.insert(k, v);
        });
        ht
    }
}

unsafe impl<K: Eq + Send, V: Send, H: Hash<K>, A: Allocator + Send> Send for HashTable<K, V, H, A> {}
unsafe impl<K: Eq + Sync, V: Sync, H: Hash<K>, A: Allocator + Sync> Sync for HashTable<K, V, H, A> {}

#[cfg(test)]
mod test {
    use memoffset::offset_of;

    use crate::hash::{DefaultHash, Hash};

    use super::HashTable;

    #[test]
    fn layout() {
        assert_eq!(
            offset_of!(HashTable<u32, u32>, bucket_array),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(HashTable<u32, u32>, bucket_count),
            std::mem::size_of::<usize>() * 2
        );
        assert_eq!(
            offset_of!(HashTable<u32, u32>, element_count),
            std::mem::size_of::<usize>() * 2 + std::mem::size_of::<u32>()
        );
        assert_eq!(
            offset_of!(HashTable<u32, u32>, rehash_policy),
            std::mem::size_of::<usize>() * 3
        );
        assert_eq!(
            offset_of!(HashTable<u32, u32>, allocator),
            std::mem::size_of::<usize>() * 4 + std::mem::size_of::<u32>()
        );
        assert_eq!(
            std::mem::size_of::<HashTable<u32, u32>>(),
            std::mem::size_of::<usize>() * 5
        );
    }

    #[test]
    fn default() {
        let ht: HashTable<u32, u32> = HashTable::default();
        assert!(ht.get(&5).is_none());
        assert!(ht.is_empty());
    }

    #[test]
    fn insert() {
        let mut ht = HashTable::new();
        ht.insert(5, 6);
        ht.insert(6, 7);
        ht.insert(4, 7);
        assert!(!ht.is_empty());
        assert_eq!(ht.len(), 3);
        assert_eq!(ht.get(&5), Some((&5, &6)));
        assert_eq!(ht.get(&6), Some((&6, &7)));
        assert_eq!(ht.get(&7), None);
    }

    #[test]
    fn remove() {
        let mut ht = HashTable::new();
        ht.insert(6, 7);
        assert_eq!(ht.remove(&6), Some(7));
        assert!(ht.is_empty());
        assert_eq!(ht.get(&6), None);
    }

    #[test]
    fn clear() {
        let mut ht = HashTable::new();
        ht.insert(1, 2);
        ht.insert(2, 3);
        ht.insert(3, 4);
        ht.clear();
        assert!(ht.is_empty());
    }

    #[test]
    fn from_iter() {
        let mut ht: HashTable<u32, u32> = [(1, 2), (2, 3), (3, 4)].into_iter().collect();
        assert_eq!(ht.len(), 3);
        assert_eq!(ht.get(&2), Some((&2, &3)));
        ht.get_mut(&3).map(|v| *v = 5);
        assert_eq!(ht.get(&3), Some((&3, &5)));
    }

    struct Test<'a> {
        a: u32,
        r: &'a mut u32,
    }

    impl<'a> Drop for Test<'a> {
        fn drop(&mut self) {
            *self.r *= 2;
        }
    }

    impl<'a> PartialEq for Test<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.a == other.a
        }
        fn ne(&self, other: &Self) -> bool {
            self.a != other.a
        }
    }
    impl<'a> Eq for Test<'a> {}

    impl<'a> Hash<Test<'a>> for DefaultHash<Test<'a>> {
        fn hash(val: &Test<'a>) -> usize {
            unsafe { std::mem::transmute(val.r as *const u32) }
        }
    }

    #[test]
    fn drop() {
        let mut foo = 1;
        let mut bar = 1;
        let mut baz = 1;
        let mut bag = 1;
        {
            let mut ht = HashTable::new();
            ht.insert(Test { a: 1, r: &mut foo }, None);
            ht.insert(Test { a: 2, r: &mut bar }, Some(Test { a: 3, r: &mut baz }));
            ht.remove(&Test { a: 2, r: &mut bag });
        }
        assert_eq!(foo, 2);
        assert_eq!(bar, 2);
        assert_eq!(baz, 2);
        assert_eq!(bag, 2);
    }

    #[derive(Debug, PartialEq, Eq)]
    struct A {
        a: u32,
    }

    impl Hash<A> for DefaultHash<A> {
        fn hash(_: &A) -> usize {
            1
        }
    }

    #[test]
    fn collisions() {
        let ht: HashTable<A, u32> = (0..11).map(|n| (A { a: n }, n)).collect();
        for i in 0..11 {
            assert_eq!(ht.get(&A { a: i }), Some((&A { a: i }, &i)));
        }
    }
}
