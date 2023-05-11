use crate::allocator::{Allocator, DefaultAllocator};
use crate::compare::{Compare, Less};
use crate::vector::Vector;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use superslice::Ext;

/// A vector map is a map backed by a vector, maintaining an order
#[repr(C)]
pub struct VectorMap<K: Eq, V, C: Compare<K> = Less<K>, A: Allocator = DefaultAllocator> {
    base: Vector<(K, V), A>,
    _phantom: PhantomData<C>,
}

impl<K: Eq + PartialOrd, V> VectorMap<K, V, Less<K>, DefaultAllocator> {
    /// Creates a new empty vector map
    pub fn new() -> Self {
        Self {
            base: Vector::new(),
            _phantom: PhantomData::default(),
        }
    }

    /// Creates a new vector map with a capacity allocated
    ///
    /// # Arguments
    ///
    /// `capacity`: The initial capacity of the vector
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: Vector::with_capacity(capacity),
            _phantom: Default::default(),
        }
    }
}

impl<K: Eq, V, C: Compare<K>, A: Allocator> VectorMap<K, V, C, A> {
    /// Returns the capacity of the vector map
    pub fn capacity(&self) -> usize {
        self.base.capacity()
    }

    /// Clears the hash map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.base.clear()
    }

    /// Checks if the hash map contains the given key
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
    pub fn get(&self, key: &K) -> Option<&V> {
        let lower_bound = self.lower_bound(key);

        // make sure the bound is in-range
        if lower_bound < self.len() {
            let (k, v) = &self.base[lower_bound];

            if k == key {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Fetches the associated value for a key
    ///
    /// # Arguments
    ///
    /// `key`: The key to search for
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let lower_bound = self.lower_bound(key);

        // make sure the bound is in-range
        if lower_bound < self.len() {
            let (k, v) = &mut self.base[lower_bound];

            if k == key {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Inserts the key-value pair into the vector map, returning the old value
    ///
    /// # Arguments
    ///
    /// `key`: The key with which to insert the pair
    ///
    /// `value`: The associated value
    pub fn insert(&mut self, key: K, mut value: V) -> Option<V> {
        // find the insertion point
        let lower_bound = self.lower_bound(&key);

        // if it already exists, just replace the value and return the original
        if lower_bound < self.len() && self.base[lower_bound].0 == key {
            std::mem::swap(&mut value, &mut self.base[lower_bound].1);

            Some(value)
        } else {
            // simply insert at the index
            self.base.insert(lower_bound, (key, value));

            None
        }
    }

    /// Returns true if the hash map is empty
    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    /// Returns the number of key-value pairs in the hash map
    pub fn len(&self) -> usize {
        self.base.len()
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
            base: Vector::new_in(allocator),
            _phantom: PhantomData::default(),
        }
    }

    /// Removes a key-value pair from the hash map,
    /// returning the element if it was found
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Removes a key-value pair from the hash map,
    /// returning the pair if it was found
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        // find the entry
        let lower_bound = self.lower_bound(key);

        if lower_bound < self.len() && &self.base[lower_bound].0 == key {
            self.base.remove(lower_bound)
        } else {
            None
        }
    }

    /// Finds the index of the first value which is not smaller
    fn lower_bound(&self, key: &K) -> usize {
        self.base.as_slice().lower_bound_by(|(k, _)| {
            // we don't perform an equality check here because we shouldn't need to. in a
            // lower bound, equal and less are the same thing
            if C::compare(k, key) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    }
}

impl<K: Eq, V, C: Compare<K>, A: Allocator> AsRef<[(K, V)]> for VectorMap<K, V, C, A> {
    fn as_ref(&self) -> &[(K, V)] {
        self.base.as_ref()
    }
}

impl<K: Eq + Debug, V: Debug, C: Compare<K>, A: Allocator> Debug for VectorMap<K, V, C, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.as_ref()
                .iter()
                .map(|(k, v)| format!("{k:?}: {v:?}"))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl<K: Eq + PartialOrd, V> Default for VectorMap<K, V, Less<K>, DefaultAllocator> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Debug, V: Debug, C: Compare<K>, A: Allocator> Deref for VectorMap<K, V, C, A> {
    type Target = [(K, V)];

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<K: Clone + Eq + PartialOrd, V: Clone> From<&[(K, V)]>
    for VectorMap<K, V, Less<K>, DefaultAllocator>
{
    fn from(value: &[(K, V)]) -> Self {
        let mut vec = VectorMap::with_capacity(value.len());
        value.iter().cloned().for_each(|(k, v)| {
            vec.insert(k, v);
        });
        vec
    }
}

impl<K: Clone + Eq + PartialOrd, V: Clone> From<&mut [(K, V)]>
    for VectorMap<K, V, Less<K>, DefaultAllocator>
{
    fn from(value: &mut [(K, V)]) -> Self {
        VectorMap::from(&*value)
    }
}

impl<K: Eq + PartialOrd, V, const N: usize> From<[(K, V); N]>
    for VectorMap<K, V, Less<K>, DefaultAllocator>
{
    fn from(value: [(K, V); N]) -> Self {
        let mut vec = VectorMap::with_capacity(value.len());
        value.into_iter().for_each(|(k, v)| {
            vec.insert(k, v);
        });
        vec
    }
}

impl<K: Clone + Eq + PartialOrd, V: Clone, const N: usize> From<&[(K, V); N]>
    for VectorMap<K, V, Less<K>, DefaultAllocator>
{
    fn from(value: &[(K, V); N]) -> Self {
        VectorMap::from(value.as_slice())
    }
}

impl<K: Eq + PartialOrd, V> FromIterator<(K, V)> for VectorMap<K, V, Less<K>, DefaultAllocator> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        // we need to insert individually here to uphold the ordering constraints
        let mut vec = Self::default();
        iter.into_iter().for_each(|(k, v)| {
            vec.insert(k, v);
        });
        vec
    }
}

unsafe impl<K: Eq + Send, V: Send, C: Compare<K> + Send, A: Allocator + Send> Send
    for VectorMap<K, V, C, A>
{
}
unsafe impl<K: Eq + Sync, V: Sync, C: Compare<K> + Sync, A: Allocator + Sync> Sync
    for VectorMap<K, V, C, A>
{
}

#[cfg(test)]
mod test {
    use crate::vector_map::VectorMap;

    #[test]
    fn layout() {
        assert_eq!(
            std::mem::size_of::<VectorMap<u32, u32>>(),
            std::mem::size_of::<usize>() * 4
        );
    }

    #[test]
    fn default_state() {
        let vec: VectorMap<u32, ()> = VectorMap::default();

        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn insert() {
        let mut vec = VectorMap::default();

        vec.insert(5, 6);

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.capacity(), 1);
        assert_eq!(&*vec, &[(5, 6)]);
    }

    #[test]
    fn from_iter() {
        let vec: VectorMap<_, _, _, _> = [(5, 6)].into_iter().collect();

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.capacity(), 1);
        assert_eq!(&*vec, &[(5, 6)]);
    }

    #[test]
    fn from_owned() {
        let vec = VectorMap::from([(5, 6)]);

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.capacity(), 1);
        assert_eq!(&*vec, &[(5, 6)]);
    }

    #[test]
    fn from_ref() {
        let vec = VectorMap::from(&[(5, 6)]);

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.capacity(), 1);
        assert_eq!(&*vec, &[(5, 6)]);
    }

    #[test]
    fn get() {
        let vec = VectorMap::from([(5, 6)]);

        assert_eq!(vec.get(&5), Some(&6));
        assert_eq!(vec.get(&6), None);
    }

    #[test]
    fn get_mut() {
        let mut vec = VectorMap::from([(5, 6)]);

        let val = vec.get_mut(&5).unwrap();
        assert_eq!(val, &mut 6);

        // update the value
        *val = 7;
        assert_eq!(val, &mut 7);
        assert_eq!(vec.get(&5), Some(&7));
        assert_eq!(vec.get_mut(&6), None);
    }

    #[test]
    fn insert_less() {
        let mut vec = VectorMap::default();

        vec.insert(5, 6);
        vec.insert(4, 5);

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 2);
        assert_eq!(&*vec, &[(4, 5), (5, 6)]);
    }

    #[test]
    fn iter() {
        let vec = VectorMap::from([(5, 6), (4, 7)]);

        assert_eq!(vec.iter().next().unwrap().1, 7);
        assert_eq!(vec.iter().len(), 2);
    }

    #[test]
    fn big_test() {
        let vec: VectorMap<_, _> = (0..50)
            .map(|x| x * 2)
            .chain((0..50).map(|x| x * 2 + 1))
            .map(|x| (x, x + 2))
            .collect();

        // make sure the vec is sorted
        assert!(vec.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(vec.len(), 100);
    }
}
