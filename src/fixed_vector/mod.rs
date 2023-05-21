//!
//! Copyright (C) Warsaw Revamped. Any unauthorized use, modification, or distribution of any portion of this file is prohibited. All rights reserved.
//!

use crate::allocator::Allocator;
use crate::fixed_vector::allocator::FixedVectorAllocator;
use crate::vector::Vector;

mod allocator;

/// TODO overflow_enabled, more than traversal?

#[repr(C)]
pub struct FixedVector<T: Sized, const NODE_COUNT: usize, A: Allocator> {
    base_vec: Vector<T, FixedVectorAllocator<A>>,
    buffer: [T; NODE_COUNT],
}

impl<T: Sized, const NODE_COUNT: usize, A: Allocator> FixedVector<T, NODE_COUNT, A> {
    /// Returns the max fixed size, which is the user-supplied NodeCount parameter
    pub fn max_size(&self) -> usize {
        NODE_COUNT
    }

    /// Returns true if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the vector
    pub fn len(&self) -> usize {
        self.base_vec.len()
    }

    /// Returns true if the fixed space has been fully allocated. Note that if overflow is enabled, the container size can be greater than nodeCount but full() could return true because the fixed space may have a recently freed slot.
    pub fn is_full(&self) -> bool {
        // If len >= capacity (NodeCount), then we are definitely full.
        // Also, if our size is smaller but we've switched away from self.buffer due to a previous overflow, then we are considered full.
        self.base_vec.len() >= NODE_COUNT
            || self.base_vec.begin_ptr.cast_const() != self.buffer.as_ptr()
    }
}
