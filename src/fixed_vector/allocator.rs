use crate::allocator::Allocator;
use std::ffi::c_void;
use std::ptr;
use std::ptr::null_mut;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FixedVectorAllocator<A: Allocator> {
    overflow_allocator: A,
    pub pool_begin: *mut c_void,
}

impl<A: Allocator> FixedVectorAllocator<A> {
    /// Creates a fixed vector allocator with a custom overflow allocator
    ///
    /// # Arguments
    /// `overflow_allocator`: The allocator to use when the fixed vector overflows
    pub fn new_with(overflow_allocator: A) -> Self {
        Self {
            overflow_allocator,
            pool_begin: null_mut(),
        }
    }
}

impl<A: Allocator + Default> Default for FixedVectorAllocator<A> {
    fn default() -> Self {
        Self {
            overflow_allocator: A::default(),
            pool_begin: null_mut(),
        }
    }
}

unsafe impl<A: Allocator> Allocator for FixedVectorAllocator<A> {
    fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut () {
        self.overflow_allocator.allocate_raw_aligned(n, align)
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), n: usize, align: usize) {
        if !ptr::eq(p, self.pool_begin.cast()) {
            self.overflow_allocator.deallocate_raw_aligned(p, n, align)
        }
    }
}

unsafe impl<A: Allocator + Send> Send for FixedVectorAllocator<A> {}
unsafe impl<A: Allocator + Sync> Sync for FixedVectorAllocator<A> {}
