pub mod with_overflow;

use crate::allocator::Allocator;
use std::marker::PhantomData;
use std::{mem, ptr};

pub trait PoolAllocator: Allocator {
    /// Initializes the fixed pool allocator with the given memory.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    unsafe fn init(&mut self, memory: &mut [u8]);
}

/// The struct `eastl::fixed_pool_base::Link`. Singly-linked list for memory allocations.
pub(crate) struct Link {
    next: *mut Link,
}

/// The class `eastl::fixed_pool`. Allocates out of a block of memory.
#[repr(C)]
pub struct FixedPool<Node: Sized> {
    pub(crate) head: *mut Link,
    pub(crate) next: *mut Link,
    pub(crate) capacity: *mut Link,
    _node: PhantomData<Node>,
}

impl<Node: Sized> FixedPool<Node> {
    /// Creates the fixed pool allocator with the given memory.
    ///
    /// # Safety
    /// `memory` must be a valid chunk of memory, solely owned and managed by the pool allocator.
    #[allow(dead_code)]
    pub unsafe fn new(memory: &mut [u8]) -> Self {
        let mut res = Self::default();
        res.init(memory);
        res
    }

    /// Returns true if the pool can allocate.
    #[allow(dead_code)]
    pub fn can_allocate(&self) -> bool {
        !self.head.is_null() || (self.next != self.capacity)
    }
}

impl<Node: Sized> PoolAllocator for FixedPool<Node> {
    unsafe fn init(&mut self, memory: &mut [u8]) {
        let mut memory_size = memory.len();
        let mut node_align = mem::align_of::<Node>();
        let node_size = mem::size_of::<Node>();

        // make sure node alignment is a power of two (or 1)
        debug_assert!(
            (node_align & (node_align - 1)) == 0,
            "Node alignment must be a power of 2 (or 1)"
        );

        // node alignment must be at least 1
        if node_align < 1 {
            node_align = 1;
        }

        let next = (memory.as_ptr() as usize + (node_align - 1)) & !(node_align - 1);
        // chop off any bytes consumed by alignment requirements
        memory_size -= next - memory.as_ptr() as usize;

        // this isn't really a valid case, but we'll add a debug check anyway
        debug_assert!(node_size >= mem::size_of::<Link>());

        // chop down the end of the memory block to fit size requirements
        memory_size = (memory_size / node_size) * node_size;

        self.head = ptr::null_mut();
        self.next = next as *mut Link;
        self.capacity = (next + memory_size) as *mut Link;
    }
}

unsafe impl<Node: Sized> Allocator for FixedPool<Node> {
    fn allocate_raw_aligned(&mut self, _n: usize, _align: usize) -> *mut () {
        debug_assert!(_n == mem::size_of::<Node>());
        debug_assert!(_align == mem::align_of::<Node>());

        let mut link = self.head;

        if let Some(link) = unsafe { link.as_mut() } {
            // if there's an already-freed node...
            self.head = link.next;
            (link as *mut Link).cast()
        } else {
            // there aren't any freed nodes. allocate a new one
            if self.next != self.capacity {
                link = self.next;
                self.next = unsafe { self.next.cast::<u8>().add(mem::size_of::<Node>()) }.cast();
                link.cast()
            } else {
                ptr::null_mut()
            }
        }
    }

    unsafe fn deallocate_raw_aligned(&mut self, p: *mut (), _n: usize, _align: usize) {
        let link = p.cast::<Link>();

        // of course the link needs to be valid
        debug_assert!(!p.is_null());

        // add to the linked list
        unsafe { &mut *link }.next = self.head;
        self.head = link;
    }
}

impl<Node: Sized> Default for FixedPool<Node> {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            next: ptr::null_mut(),
            capacity: ptr::null_mut(),
            _node: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::allocator::Allocator;
    use crate::fixed_pool::FixedPool;
    use memoffset::offset_of;
    use std::mem;

    #[repr(C, align(0x10))]
    struct TestNode {
        a: usize,
    }

    #[test]
    fn layout() {
        assert_eq!(offset_of!(FixedPool<TestNode>, head), 0);
        assert_eq!(
            offset_of!(FixedPool<TestNode>, next),
            mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(FixedPool<TestNode>, capacity),
            mem::size_of::<usize>() * 2
        );

        assert_eq!(
            mem::size_of::<FixedPool<TestNode>>(),
            mem::size_of::<usize>() * 3
        );
    }

    #[test]
    fn simple_alloc_happy_case() {
        let mut buf = [0; mem::size_of::<TestNode>() * 2];

        let mut allocator = unsafe { FixedPool::<TestNode>::new(&mut buf) };
        assert!(allocator.can_allocate());

        let p: *mut TestNode = allocator.allocate(1);

        // ensure alignment and size constraints are met
        assert!(!p.is_null());
        assert_eq!((p as usize & (mem::align_of::<TestNode>() - 1)), 0);
        assert!(p as usize + mem::size_of::<TestNode>() <= allocator.capacity as usize);
    }

    #[test]
    fn simple_alloc_empty() {
        // only space for one node
        let mut buf = [0; (mem::size_of::<TestNode>() * 2) - 1];

        let mut allocator = unsafe { FixedPool::<TestNode>::new(&mut buf) };
        assert!(allocator.can_allocate());

        // allocate the only possible node
        let _: *mut TestNode = allocator.allocate(1);

        // now we cannot allocate
        assert!(!allocator.can_allocate());
        let p: *mut TestNode = allocator.allocate(1);

        assert!(p.is_null());
    }

    #[test]
    fn simple_alloc_realloc() {
        // only space for one node
        let mut buf = [0; (mem::size_of::<TestNode>() * 2) - 1];

        let mut allocator = unsafe { FixedPool::<TestNode>::new(&mut buf) };
        assert!(allocator.can_allocate());

        // allocate the only possible node
        let p: *mut TestNode = allocator.allocate(1);
        assert!(!allocator.can_allocate());

        // de-allocate the node
        unsafe { allocator.deallocate(p, 1) };

        // we should be able to allocate again
        assert!(allocator.can_allocate());
        let p: *mut TestNode = allocator.allocate(1);

        assert!(!p.is_null());
        assert_eq!((p as usize & (mem::align_of::<TestNode>() - 1)), 0);
        assert!(p as usize + mem::size_of::<TestNode>() <= allocator.capacity as usize);
    }
}
