//!
//! Copyright (C) Warsaw Revamped. Any unauthorized use, modification, or distribution of any portion of this file is prohibited. All rights reserved.
//!

use crate::Allocator;

/// Implements a consuming
pub struct IntoIter<T: Sized, A: Allocator> {
    pub(crate) begin_ptr: *mut T,
    pub(crate) capacity_ptr: *mut T,
    pub(crate) front_cursor: *mut T,
    pub(crate) back_cursor: *mut T,
    pub(crate) allocator: A,
}

impl<T: Sized, A: Allocator> IntoIter<T, A> {
    /// Returns the vector as raw bytes
    pub fn as_slice(&self) -> &[T] {
        if let Some(front_cursor) = unsafe { self.front_cursor.as_ref() } {
            unsafe { std::slice::from_raw_parts(front_cursor, self.len()) }
        } else {
            &[]
        }
    }

    /// Returns the vector as raw bytes
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        if let Some(front_cursor) = unsafe { self.front_cursor.as_mut() } {
            unsafe { std::slice::from_raw_parts_mut(front_cursor, self.len()) }
        } else {
            &mut []
        }
    }

    /// Returns the total capacity of the vector.
    pub fn capacity(&self) -> usize {
        (unsafe { self.capacity_ptr.offset_from(self.begin_ptr) }) as usize
    }

    /// Returns true if there are no elements in the vector.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the remaining slice.
    pub fn len(&self) -> usize {
        (unsafe { self.back_cursor.offset_from(self.front_cursor) }) as usize
    }
}

impl<T: Sized, A: Allocator> Drop for IntoIter<T, A> {
    fn drop(&mut self) {
        if !self.begin_ptr.is_null() {
            unsafe {
                // drop all elements in place
                std::ptr::drop_in_place(self.as_slice_mut());
                // free the array
                self.allocator
                    .deallocate::<T>(self.begin_ptr, self.capacity())
            }
        }
    }
}

impl<T: Sized, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // if we have remaining capacity
        if self.is_empty() {
            None
        } else {
            // read from the front cursor and decrement
            let next_val = unsafe { self.front_cursor.read() };
            self.front_cursor = unsafe { self.front_cursor.add(1) };
            Some(next_val)
        }
    }
}

impl<T: Sized, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // if we have remaining capacity
        if self.is_empty() {
            None
        } else {
            // decrement the end ptr and read from it
            self.back_cursor = unsafe { self.back_cursor.sub(1) };
            let next_val = unsafe { self.back_cursor.read() };
            Some(next_val)
        }
    }
}
