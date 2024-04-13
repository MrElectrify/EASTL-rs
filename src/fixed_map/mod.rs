//!
//! Copyright (C) Warsaw Revamped. Any unauthorized use, modification, or distribution of any portion of this file is prohibited. All rights reserved.
//!

use crate::allocator::{Allocator, DefaultAllocator};
use crate::compare::{Compare, Less};
use crate::fixed_pool::with_overflow::FixedPoolWithOverflow;
use crate::fixed_pool::{FixedPool, PoolAllocator};
use crate::internal::rb_tree::{
    iter::{Iter, IterMut},
    node::Node,
};
use crate::map::Map;
use duplicate::duplicate_item;
use moveit::{new, New};
use std::mem::MaybeUninit;
use std::{mem, slice};

/// A fixed map with overflow which uses the default allocator as an overflow.
pub type DefaultFixedMapWithOverflow<K, V, const NODE_COUNT: usize, C> =
    FixedMapWithOverflow<K, V, NODE_COUNT, DefaultAllocator, C>;

/// A fixed map without overflow.
pub type FixedMap<K, V, const NODE_COUNT: usize, C = Less<K>> =
    FixedMapImpl<K, V, NODE_COUNT, FixedPool<Node<K, V>>, C>;

/// A fixed map with overflow using the given overflow allocator.
pub type FixedMapWithOverflow<K, V, const NODE_COUNT: usize, OverflowAllocator, C = Less<K>> =
    FixedMapImpl<K, V, NODE_COUNT, FixedPoolWithOverflow<Node<K, V>, OverflowAllocator>, C>;

#[repr(C)]
pub struct FixedMapImpl<
    K: PartialEq,
    V,
    const NODE_COUNT: usize,
    A: Allocator,
    C: Compare<K> = Less<K>,
> {
    // real EASTL uses a fixed_node_pool here, which is just fixed_pool_with_overflow templated
    // by node size instead of type, so it does not matter and we use fixed_pool_with_overflow
    // directly
    base_map: Map<K, V, A, C>,
    // this should `technically` be conformant - `buffer` should be aligned to the alignment of
    // `ListNode<T>`...
    buffer: [MaybeUninit<Node<K, V>>; NODE_COUNT],
    // ... and then we add an extra node for the padding induced as shown in the conformant version (of FixedList)
    _pad: MaybeUninit<Node<K, V>>,
}

impl<
        K: PartialEq,
        V,
        const NODE_COUNT: usize,
        A: PoolAllocator + Default,
        C: Compare<K> + Default,
    > FixedMapImpl<K, V, NODE_COUNT, A, C>
{
    /// Create a new, empty fixed map.
    ///
    /// # Arguments
    /// `allocator`: The allocator to use
    ///
    /// # Safety
    /// The resulting map must not be moved.
    pub unsafe fn new() -> impl New<Output = Self> {
        new::of(Self {
            base_map: Map::with_allocator(A::default()),
            // we actually don't care what the buffer contains
            buffer: MaybeUninit::uninit().assume_init(),
            _pad: MaybeUninit::uninit().assume_init(),
        })
        .with(|this| {
            let this = this.get_unchecked_mut();
            this.base_map
                .inner
                .allocator
                .init(slice::from_raw_parts_mut(
                    this.buffer.as_mut_ptr().cast(),
                    this.buffer.len() * mem::size_of::<Node<K, V>>(),
                ));
        })
    }
}

impl<K: PartialEq, V, const NODE_COUNT: usize, A: Allocator, C: Compare<K>>
    FixedMapImpl<K, V, NODE_COUNT, A, C>
{
    /// Clears the map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.base_map.clear()
    }

    /// Returns true if the map contains a pair indexed
    /// by the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn contains_key(&self, key: &K) -> bool {
        self.base_map.contains_key(key)
    }

    /// Fetches the value indexed by the key in the map
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get(&self, key: &K) -> Option<&V> {
        self.base_map.get(key)
    }

    /// Fetches the value indexed by the key in the map
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.base_map.get_mut(key)
    }

    /// Returns true if the map contains no elements
    pub fn is_empty(&self) -> bool {
        self.base_map.is_empty()
    }

    /// Returns an iterator over the elements in the map.
    ///
    /// # Safety
    /// This iterator is not tested as trees are only partially implemented.
    #[duplicate_item(
    iter        Self        Iter;
    [iter]      [&Self]     [Iter];
    [iter_mut]  [&mut Self] [IterMut];
    )]
    #[allow(clippy::needless_arbitrary_self_type)]
    pub unsafe fn iter(self: Self) -> Iter<K, V> {
        self.base_map.iter()
    }

    /// Returns the number of elements in the map
    pub fn len(&self) -> usize {
        self.base_map.len()
    }

    /// Removes a key-value pair from the map,
    /// returning the pair if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        self.base_map.remove_entry(key)
    }
}
