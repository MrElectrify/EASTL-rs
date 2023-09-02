use std::alloc::{self, Layout};

/// An object which allocates memory for use.
///
/// # Safety
///
/// The implementor must ensure that `n` is non-zero, and that the pointers returned are the
/// specified size and alignment.
pub unsafe trait Allocator {
    /// Allocate an array of `n` items. `n` must not be zero.
    ///
    /// # Arguments
    ///
    /// `n`: The number of array elements
    fn allocate<T>(&mut self, n: usize) -> *mut T {
        unsafe {
            std::mem::transmute(
                self.allocate_raw_aligned(n * std::mem::size_of::<T>(), std::mem::align_of::<T>()),
            )
        }
    }

    /// Allocate `n` bytes aligned to usize. `n` must not be zero.
    ///
    /// # Arguments
    ///
    /// `n`: The number of bytes to allocate
    fn allocate_raw(&mut self, n: usize) -> *mut () {
        self.allocate_raw_aligned(n, std::mem::size_of::<usize>())
    }

    /// Allocate `n` bytes aligned to `align` bytes. `n` must not be zero.
    ///
    /// # Arguments
    ///
    /// `n`: The number of bytes to allocate
    ///
    /// `align`: The alignment of the block to allocate
    fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut ();

    /// Deallocates the block `p` of size `n` bytes aligned to usize and returns it to
    /// available memory to re-allocate
    ///
    /// # Safety
    ///
    /// `p` must be a valid pointer to an array with size `n`.
    unsafe fn deallocate<T>(&mut self, p: *mut T, n: usize) {
        self.deallocate_raw_aligned(
            std::mem::transmute(p),
            n * std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
        )
    }

    /// Deallocates the block `p` of size `n` bytes aligned to usize and returns it to
    /// available memory to re-allocate
    ///
    /// # Arguments
    ///
    /// `p`: The pointer to the block of memory
    ///
    /// `n`: The number of bytes to deallocate
    ///
    /// # Safety
    ///
    /// `p` must be a valid pointer
    unsafe fn deallocate_raw(&mut self, p: *mut (), n: usize) {
        self.deallocate_raw_aligned(p, n, std::mem::size_of::<usize>())
    }

    /// Deallocates the block `p` of size `n` bytes and returns it to
    /// available memory to re-allocate
    ///
    /// # Arguments
    ///
    /// `p`: The pointer to the block of memory
    ///
    /// `n`: The number of bytes to deallocate
    ///
    /// `align`: The alignment of the block of memory
    ///
    /// # Safety
    ///
    /// `p` must be a valid pointer
    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), n: usize, align: usize);
}

#[derive(Default)]
pub struct DefaultAllocator {
    // padding due to 1-size struct in C
    _dummy: u8,
}

unsafe impl Allocator for DefaultAllocator {
    fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut () {
        assert_ne!(n, 0, "`n` must not be zero!");

        unsafe {
            std::mem::transmute(alloc::alloc(
                Layout::array::<u8>(n).unwrap().align_to(align).unwrap(),
            ))
        }
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), n: usize, align: usize) {
        alloc::dealloc(
            std::mem::transmute(p),
            Layout::array::<u8>(n).unwrap().align_to(align).unwrap(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Allocator, DefaultAllocator};

    #[test]
    fn layout() {
        assert_eq!(
            std::mem::size_of::<DefaultAllocator>(),
            std::mem::size_of::<u8>()
        )
    }

    #[test]
    fn align() {
        let mut alloc = DefaultAllocator::default();
        let aligned_by_4 = unsafe { alloc.allocate_raw_aligned(20, 4) };
        unsafe { alloc.deallocate_raw_aligned(aligned_by_4, 20, 4) };
        let aligned_by_8 = unsafe { alloc.allocate_raw_aligned(20, 8) };
        unsafe { alloc.deallocate_raw_aligned(aligned_by_8, 20, 8) };
        let aligned_by_16 = unsafe { alloc.allocate_raw_aligned(20, 16) };
        unsafe { alloc.deallocate_raw_aligned(aligned_by_16, 20, 16) };
        assert_eq!((aligned_by_4 as usize) % 4, 0);
        assert_eq!((aligned_by_8 as usize) % 8, 0);
        assert_eq!((aligned_by_16 as usize) % 16, 0);
    }
}
