use crate::allocator::Allocator;

pub(crate) struct NullTerminatorAllocator<A: Allocator>(pub(crate) A);

unsafe impl<A: Allocator> Allocator for NullTerminatorAllocator<A> {
    fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut () {
        // always leave room for the null terminator
        let res = self.0.allocate_raw_aligned(n + 1, align);
        // null terminate
        *(unsafe { &mut *(res as *mut u8).add(n) }) = 0;
        res
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), n: usize, align: usize) {
        // always leave room for the null terminator
        self.0.deallocate_raw_aligned(p, n + 1, align)
    }
}

impl<A: Allocator + Clone> Clone for NullTerminatorAllocator<A> {
    fn clone(&self) -> Self {
        NullTerminatorAllocator(self.0.clone())
    }
}

impl<A: Allocator + Default> Default for NullTerminatorAllocator<A> {
    fn default() -> Self {
        Self(A::default())
    }
}
