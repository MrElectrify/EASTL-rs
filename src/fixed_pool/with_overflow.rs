use crate::allocator::Allocator;
use crate::fixed_pool::FixedPool;
use std::{mem, ptr};

/// The class `eastl::fixed_pool_with_overflow`. Attempts to allocate from the pool,
/// but allocates from the overflow allocator if no space is left in the pool.
#[repr(C)]
pub struct FixedPoolWithOverflow<Node: Sized, OverflowAllocator: Allocator> {
    pub(crate) pool_allocator: FixedPool<Node>,
    overflow_allocator: OverflowAllocator,
    pub(crate) pool_begin: *mut (),
}

impl<Node: Sized, OverflowAllocator: Allocator> FixedPoolWithOverflow<Node, OverflowAllocator> {
    /// Creates the fixed pool allocator with the given memory and overflow allocator.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    #[allow(dead_code)]
    pub unsafe fn new(memory: &mut [u8], overflow_allocator: OverflowAllocator) -> Self {
        let mut res = Self::with_allocator(overflow_allocator);
        res.init(memory);
        res
    }

    /// Creates an uninitialized pool allocator given the overflow allocator.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    pub unsafe fn with_allocator(overflow_allocator: OverflowAllocator) -> Self {
        Self {
            pool_allocator: FixedPool::default(),
            overflow_allocator,
            pool_begin: ptr::null_mut(),
        }
    }

    /// Returns true if the underlying pool allocator can allocate. If this method returns false,
    /// we can still allocate, but will allocate with the overflow allocator.
    #[allow(dead_code)]
    pub fn can_allocate(&self) -> bool {
        self.pool_allocator.can_allocate()
    }

    /// Initializes the fixed pool allocator with the given memory.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    pub unsafe fn init(&mut self, memory: &mut [u8]) {
        self.pool_allocator.init(memory);

        // store the pool base
        self.pool_begin = memory.as_mut_ptr().cast();
    }
}

impl<Node: Sized, OverflowAllocator: Allocator + Default>
    FixedPoolWithOverflow<Node, OverflowAllocator>
{
    /// Creates the fixed pool allocator with the given memory and the default overflow allocator.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    #[allow(dead_code)]
    pub unsafe fn new_with_default_allocator(memory: &mut [u8]) -> Self {
        Self::new(memory, OverflowAllocator::default())
    }
}

unsafe impl<Node: Sized, OverflowAllocator: Allocator> Allocator
    for FixedPoolWithOverflow<Node, OverflowAllocator>
{
    fn allocate_raw_aligned(&mut self, n: usize, align: usize) -> *mut () {
        debug_assert!(n == mem::size_of::<Node>());
        debug_assert!(align == mem::align_of::<Node>());

        // try the pool first...
        let p = self.pool_allocator.allocate_raw_aligned(n, align);
        if !p.is_null() {
            p
        } else {
            self.overflow_allocator.allocate_raw_aligned(n, align)
        }
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), _n: usize, _align: usize) {
        // if it's contained within the pool allocator, use the pool allocator
        if self.pool_begin <= p && p <= self.pool_allocator.capacity.cast() {
            self.pool_allocator.deallocate_raw_aligned(p, _n, _align)
        } else {
            self.overflow_allocator
                .deallocate_raw_aligned(p, _n, _align)
        }
    }
}

impl<Node: Sized, OverflowAllocator: Allocator + Default> Default
    for FixedPoolWithOverflow<Node, OverflowAllocator>
{
    fn default() -> Self {
        Self {
            pool_allocator: FixedPool::default(),
            overflow_allocator: OverflowAllocator::default(),
            pool_begin: ptr::null_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::allocator::{Allocator, DefaultAllocator};
    use crate::fixed_pool::with_overflow::FixedPoolWithOverflow;
    use crate::fixed_pool::FixedPool;
    use memoffset::offset_of;
    use std::mem;

    #[repr(C, align(0x10))]
    struct TestNode {
        a: usize,
    }

    #[test]
    fn layout() {
        assert_eq!(
            offset_of!(FixedPoolWithOverflow<TestNode, DefaultAllocator>, pool_allocator),
            0
        );
        assert_eq!(
            offset_of!(FixedPoolWithOverflow<TestNode, DefaultAllocator>, overflow_allocator),
            mem::size_of::<FixedPool<TestNode>>()
        );
        assert_eq!(
            offset_of!(FixedPoolWithOverflow<TestNode, DefaultAllocator>, pool_begin),
            mem::size_of::<FixedPool<TestNode>>() + mem::size_of::<usize>()
        );

        assert_eq!(
            mem::size_of::<FixedPoolWithOverflow<TestNode, DefaultAllocator>>(),
            mem::size_of::<FixedPool<TestNode>>() + mem::size_of::<usize>() * 2
        );
    }

    #[test]
    fn simple_alloc_happy_case() {
        let mut buf = [0; mem::size_of::<TestNode>() * 2];

        let mut allocator = unsafe {
            FixedPoolWithOverflow::<TestNode, _>::new(&mut buf, DefaultAllocator::default())
        };
        assert!(allocator.can_allocate());

        let p: *mut TestNode = allocator.allocate(1);

        // ensure alignment and size constraints are met
        assert!(!p.is_null());
        assert_eq!((p as usize & (mem::align_of::<TestNode>() - 1)), 0);
        assert!(
            p as usize + mem::size_of::<TestNode>() <= allocator.pool_allocator.capacity as usize
        );
    }

    #[test]
    fn simple_alloc_overflow() {
        // only space for one node
        let mut buf = [0; (mem::size_of::<TestNode>() * 2) - 1];

        let mut allocator = unsafe {
            FixedPoolWithOverflow::<TestNode, _>::new(&mut buf, DefaultAllocator::default())
        };
        assert!(allocator.can_allocate());

        // allocate the only possible node
        let _: *mut TestNode = allocator.allocate(1);

        // now we cannot allocate
        assert!(!allocator.can_allocate());
        let p: *mut TestNode = allocator.allocate(1);

        // ensure alignment and size constraints are met
        assert!(!p.is_null());
        assert_eq!((p as usize & (mem::align_of::<TestNode>() - 1)), 0);

        // make sure the allocation is not within the pool
        assert!(p < allocator.pool_begin.cast() || p >= allocator.pool_allocator.capacity.cast());
    }

    #[test]
    fn simple_alloc_realloc() {
        // only space for one node
        let mut buf = [0; (mem::size_of::<TestNode>() * 2) - 1];

        let mut allocator = unsafe {
            FixedPoolWithOverflow::<TestNode, _>::new(&mut buf, DefaultAllocator::default())
        };
        assert!(allocator.can_allocate());

        // allocate the only possible node
        let pool_allocated: *mut TestNode = allocator.allocate(1);
        assert!(!pool_allocated.is_null());

        // now we cannot allocate
        assert!(!allocator.can_allocate());
        let overflow_allocated: *mut TestNode = allocator.allocate(1);
        // make sure the allocation is not within the pool
        assert!(
            overflow_allocated < allocator.pool_begin.cast()
                || overflow_allocated >= allocator.pool_allocator.capacity.cast()
        );

        // deallocate the overflow allocated first
        unsafe { allocator.deallocate(overflow_allocated, 1) };
        // make sure that we still can't allocate
        assert!(!allocator.can_allocate());

        // deallocate the pool allocated value now
        unsafe { allocator.deallocate(pool_allocated, 1) };
        // we should be able to allocate now
        assert!(allocator.can_allocate());
    }
}
