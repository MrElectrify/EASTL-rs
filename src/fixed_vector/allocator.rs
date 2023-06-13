use crate::allocator::Allocator;
use std::ffi::c_void;

#[repr(C)]
pub struct FixedVectorAllocator<A: Allocator> {
    overflow_allocator: A,
    pool_begin: *mut c_void,
}

impl<A: Allocator> Allocator for FixedVectorAllocator<A> {
    unsafe fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut () {
        self.overflow_allocator.allocate_raw_aligned(n, align)
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), n: usize, align: usize) {
        if p.cast() != self.pool_begin {
            self.overflow_allocator.deallocate_raw_aligned(p, n, align)
        }
    }
}
