use crate::allocator::{Allocator, DefaultAllocator};
use crate::fixed_vector::allocator::FixedVectorAllocator;
use crate::vector::Vector;
use moveit::new::New;
use moveit::{new, MoveNew, MoveRef};
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::{mem, ptr};

mod allocator;

/// TODO overflow_enabled, more than traversal?

/// Fixed vector with the default allocator.
pub type DefaultFixedVector<T, const NODE_COUNT: usize> =
    FixedVector<T, NODE_COUNT, DefaultAllocator>;

#[repr(C)]
pub struct FixedVector<T: Sized + Default + Copy, const NODE_COUNT: usize, A: Allocator> {
    base_vec: Vector<T, FixedVectorAllocator<A>>,
    buffer: [T; NODE_COUNT],
}

impl<T: Sized + Default + Copy, const NODE_COUNT: usize, A: Allocator>
    FixedVector<T, NODE_COUNT, A>
{
    /// Create a new fixed_vector with the given overflow allocator
    ///
    /// # Arguments
    /// `overflow_allocator`: The allocator to use for allocating overflowed elements in the base vector
    ///
    /// # Safety
    /// Raw pointer math
    pub unsafe fn new_in(overflow_allocator: A) -> impl New<Output = Self> {
        new::of(Self {
            base_vec: Vector::new_in(FixedVectorAllocator::new_with(overflow_allocator)),
            buffer: [T::default(); NODE_COUNT],
        })
        .with(|this| {
            let this = this.get_unchecked_mut();
            this.init_base_vec();
        })
    }

    fn init_base_vec(&mut self) {
        self.base_vec.begin_ptr = &mut self.buffer[0] as *mut T;
        self.base_vec.end_ptr = &mut self.buffer[0] as *mut T;
        self.base_vec.capacity_ptr =
            (&mut self.buffer[0] as *mut T as usize + NODE_COUNT) as *mut T;
        self.base_vec.allocator.pool_begin = &mut self.buffer[0] as *mut T as *mut c_void;
    }
}

impl<T: Sized + Default + Copy, const NODE_COUNT: usize, A: Allocator + Default>
    FixedVector<T, NODE_COUNT, A>
{
    /// Create a new fixed_vector
    ///
    /// # Safety
    /// See `FixedVector::new_in`
    pub unsafe fn new() -> impl New<Output = Self> {
        Self::new_in(A::default())
    }
}

unsafe impl<T: Sized + Default + Copy, const NODE_COUNT: usize, A: Allocator> MoveNew
    for FixedVector<T, NODE_COUNT, A>
{
    unsafe fn move_new(mut src: Pin<MoveRef<Self>>, this: Pin<&mut MaybeUninit<Self>>) {
        let this = this.get_unchecked_mut().assume_init_mut();
        let src = src.as_mut().get_unchecked_mut();
        // Swap the allocator over
        mem::swap(&mut this.base_vec.allocator, &mut src.base_vec.allocator);
        if !src.has_overflowed() {
            // We haven't overflowed, so we need to move the buffer
            mem::swap(&mut this.buffer, &mut src.buffer);
            // ... and re-init the base vec pointers point to it
            this.init_base_vec();
        } else {
            // We have overflowed - we are not going to use `buffer` anymore so we might as well
            // leave it uninit - so we only copy over the base vec pointers
            this.base_vec.begin_ptr = src.base_vec.begin_ptr;
            this.base_vec.end_ptr = src.base_vec.end_ptr;
            this.base_vec.capacity_ptr = src.base_vec.capacity_ptr;
        }
    }
}

impl<T: Sized + Default + Copy, const NODE_COUNT: usize, A: Allocator>
    FixedVector<T, NODE_COUNT, A>
{
    /// Returns the vector as raw bytes
    pub fn as_slice(&self) -> &[T] {
        self.base_vec.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self.base_vec.as_slice_mut()
    }

    /// Returns the max fixed size, which is the user-supplied NodeCount parameter
    pub fn max_size(&self) -> usize {
        NODE_COUNT
    }

    /// Returns true if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.base_vec.is_empty()
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

    /// Returns true if the allocations spilled over into the overflow allocator. Meaningful only if overflow is enabled.
    pub fn has_overflowed(&self) -> bool {
        !ptr::eq(self.base_vec.begin_ptr, &self.buffer[0])
    }

    /// Pushes a new element into the vector
    ///
    /// # Arguments
    ///
    /// `elem`: The new element
    pub fn push(&mut self, elem: T) {
        self.base_vec.push(elem)
    }

    /// Pops an element off of the back of the array
    pub fn pop(&mut self) -> Option<T> {
        self.base_vec.pop()
    }

    /// Inserts an element into the array at an index.
    /// `index` must be less than or equal to `size`
    ///
    /// # Arguments
    ///
    /// `index`: The index to insert the element
    ///
    /// `elem`: The element to add to the array
    pub fn insert(&mut self, index: usize, elem: T) {
        self.base_vec.insert(index, elem)
    }
}

#[cfg(test)]
mod test {
    use crate::fixed_vector::DefaultFixedVector;
    use moveit::moveit;

    #[test]
    fn init_push() {
        moveit! {
            let mut fixed_vec = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        assert_eq!(fixed_vec.len(), 0);
        assert!(!fixed_vec.has_overflowed());
        fixed_vec.push(64);
        assert_eq!(fixed_vec.len(), 1);
        assert_eq!(fixed_vec.as_slice()[0], 64);
        assert!(!fixed_vec.has_overflowed());
    }

    #[test]
    fn overflow() {
        moveit! {
            let mut fixed_vec = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        for i in 0..12 {
            fixed_vec.push(i);
        }
        assert_eq!(fixed_vec.len(), 12);
        assert!(fixed_vec.has_overflowed());
        assert_eq!(fixed_vec.as_slice()[11], 11);
    }
}
