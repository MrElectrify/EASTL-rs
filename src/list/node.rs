use std::marker::PhantomData;
use std::ptr::null_mut;

#[repr(C)]
pub(crate) struct ListNodeBase<T> {
    pub(crate) next: *mut ListNodeBase<T>,
    pub(crate) prev: *mut ListNodeBase<T>,
    // let's go against the idea that this shouldn't be generic as in the original implementation
    _holds_data: PhantomData<T>,
}

impl<T> Default for ListNodeBase<T> {
    fn default() -> Self {
        Self {
            next: null_mut(),
            prev: null_mut(),
            _holds_data: PhantomData,
        }
    }
}

impl<T> ListNodeBase<T> {
    /// Inserts this node in `next`s list before `next`.
    pub(crate) unsafe fn insert(&mut self, next: *mut ListNodeBase<T>) {
        self.next = next;
        self.prev = (*next).prev;
        (*(*next).prev).next = self;
        (*next).prev = self;
    }

    // Removes this node from the list that it's in. Assumes that the
    // node is within a list and thus that its prev/next pointers are valid.
    pub(crate) unsafe fn remove(&mut self) {
        (*self.next).prev = self.prev;
        (*self.prev).next = self.next;
    }
}

#[repr(C)]
pub(crate) struct ListNode<T> {
    pub(crate) base: ListNodeBase<T>,
    pub(crate) value: T,
}

impl<T> ListNode<T> {
    /// Get a reference to the contained value
    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the contained value
    pub(crate) fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
