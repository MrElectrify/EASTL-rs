use crate::allocator::{Allocator, DefaultAllocator};
use crate::list::iter::{Iter, IterMut};
use crate::list::node::{ListNode, ListNodeBase};
use moveit::{new, New};
use std::mem::size_of;
use std::{fmt, ptr};

mod iter;
mod node;

/// List with the default allocator.
pub type DefaultList<V> = List<V, DefaultAllocator>;

/// A doubly linked list.
/// The API is modelled after `std::collections::LinkedList`.
#[repr(C)]
pub struct List<T, A: Allocator> {
    /// Sentinel node, contains the front and back node pointers (prev = back, next = front)
    node: ListNodeBase<T>,
    size: u32,
    allocator: A,
}

impl<T, A: Allocator> List<T, A> {
    /// Create a new, empty list.
    ///
    /// # Arguments
    /// `allocator`: The allocator to use
    ///
    /// # Safety
    /// The resulting list must not be moved.
    pub unsafe fn new_in(allocator: A) -> impl New<Output = Self> {
        new::of(Self {
            node: ListNodeBase::default(),
            size: 0,
            allocator,
        })
        .with(|this| {
            let this = this.get_unchecked_mut();
            this.init_sentinel_node();
        })
    }

    // Init list sentinel node
    fn init_sentinel_node(&mut self) {
        self.node.prev = &mut self.node;
        self.node.next = &mut self.node;
    }

    /// Get the list's size
    pub fn size(&self) -> usize {
        self.size as usize
    }

    /// If the list is empty or not
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    unsafe fn create_node(&mut self, value: T) -> *mut ListNode<T> {
        let node = unsafe {
            self.allocator
                .allocate::<ListNode<T>>(size_of::<ListNode<T>>())
                .as_mut()
        }
        .unwrap();
        ptr::write(node.value_mut(), value);
        node
    }

    /// Drops the given node
    unsafe fn drop_node(&mut self, node: *mut ListNodeBase<T>) {
        // Drop the value
        ptr::drop_in_place(&mut (*(node as *mut ListNode<T>)).value);
        // Deallocate the memory for the node
        self.allocator.deallocate(node, size_of::<ListNode<T>>());
    }

    /// Removes the given node, extracting its value
    unsafe fn remove_node(&mut self, node: *mut ListNodeBase<T>) -> T {
        (*node).remove();
        let value = ptr::read(&(*(node as *mut ListNode<T>)).value);
        // Deallocate the memory for the node
        self.allocator.deallocate(node, size_of::<ListNode<T>>());
        self.size -= 1;
        value
    }

    /// Get a reference to the front value, if any
    ///
    /// # Return
    /// A reference to the front value if present, `None` if the list is empty.
    pub fn front(&self) -> Option<&T> {
        if self.node.next.cast_const() != &self.node {
            // this cast works as the base is the first struct member of ListNode
            Some(unsafe { (*(self.node.next as *const ListNode<T>)).value() })
        } else {
            None
        }
    }

    /// Get a reference to the back value, if any
    ///
    /// # Return
    /// A reference to the back value if present, `None` if the list is empty.
    pub fn back(&self) -> Option<&T> {
        if self.node.prev.cast_const() != &self.node {
            // this cast works as the base is the first struct member of ListNode
            Some(unsafe { &((*(self.node.prev as *const ListNode<T>)).value) })
        } else {
            None
        }
    }

    /// Get a mutable reference to the front value, if any
    ///
    /// # Return
    /// A mutable reference to the front value if present, `None` if the list is empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.node.next.cast_const() != &self.node {
            // this cast works as the base is the first struct member of ListNode
            Some(unsafe { &mut ((*(self.node.next as *mut ListNode<T>)).value) })
        } else {
            None
        }
    }

    /// Get a mutable reference to the back value, if any
    ///
    /// # Return
    /// A mutable reference to the back value if present, `None` if the list is empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.node.prev.cast_const() != &self.node {
            // this cast works as the base is the first struct member of ListNode
            Some(unsafe { &mut ((*(self.node.prev as *mut ListNode<T>)).value) })
        } else {
            None
        }
    }

    /// Push a value to the front of the list
    pub fn push_front(&mut self, value: T) {
        unsafe {
            let new_node = self.create_node(value);
            (*new_node).base.insert(self.node.next);
        }

        self.size += 1;
    }

    /// Push a value to the back of the list
    pub fn push_back(&mut self, value: T) {
        unsafe {
            let new_node = self.create_node(value);
            (*new_node).base.insert(&mut self.node);
        }
        self.size += 1;
    }

    /// Remove all elements from this list
    pub fn clear(&mut self) {
        let mut next = self.node.next;
        unsafe {
            while next != &mut self.node {
                let to_drop = next;
                // Advance the next pointer before we delete the current node
                next = (*next).next;
                // Drop the node
                self.drop_node(to_drop);
            }
        }
        self.init_sentinel_node();
        self.size = 0;
    }

    /// Return a forward iterator for this list
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            len: self.size(),
            sentinel_node: &self.node,
            current_node: (&self.node as *const ListNodeBase<T>).cast_mut(),
            marker: Default::default(),
        }
    }

    /// Return a mutable forward iterator for this list
    pub fn iter_mut(&self) -> IterMut<'_, T> {
        IterMut {
            len: self.size(),
            sentinel_node: &self.node,
            current_node: (&self.node as *const ListNodeBase<T>).cast_mut(),
            marker: Default::default(),
        }
    }

    /// Removes the first element in the list, returning its value
    ///
    /// # Return
    /// The first value if present, `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.node.next.cast_const() != &self.node {
            let value = unsafe { self.remove_node(self.node.next) };
            Some(value)
        } else {
            None
        }
    }

    /// Removes the last element in the list, returning its value
    ///
    /// # Return
    /// The last value if present, `None` if the list is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.node.prev.cast_const() != &self.node {
            let value = unsafe { self.remove_node(self.node.prev) };
            Some(value)
        } else {
            None
        }
    }
}

impl<T, A: Allocator> Drop for List<T, A> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T: fmt::Debug, A: Allocator> fmt::Debug for List<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, A: Allocator + Default> List<T, A> {
    /// Create a new, empty list
    ///
    /// # Safety
    /// The resulting list must not be moved
    pub unsafe fn new() -> impl New<Output = Self> {
        Self::new_in(A::default())
    }
}

impl<T, A: Allocator + Default> Extend<T> for List<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item)
        }
    }
}

impl<'a, T: 'a + Copy, A: Allocator + Default> Extend<&'a T> for List<T, A> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

#[cfg(test)]
mod test {
    use crate::list::DefaultList;
    use moveit::moveit;

    #[test]
    fn empty() {
        moveit! {
            let mut list = unsafe { DefaultList::<u32>::new() };
        }
        assert!(list.empty());
    }

    #[test]
    fn size_empty() {
        moveit! {
            let mut list = unsafe { DefaultList::<u32>::new() };
        }
        assert_eq!(list.size(), 0);
    }

    #[test]
    fn front_empty() {
        moveit! {
            let mut list = unsafe { DefaultList::<u32>::new() };
        }
        assert_eq!(list.front(), None)
    }

    #[test]
    fn back_empty() {
        moveit! {
            let mut list = unsafe { DefaultList::<u32>::new() };
        }
        assert_eq!(list.back(), None)
    }

    #[test]
    fn push_front() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        list.push_front(12u32);
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&12u32));
        assert_eq!(list.back(), Some(&12u32));
        list.push_front(6u32);
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&6u32));
        assert_eq!(list.back(), Some(&12u32));
    }

    #[test]
    fn push_front_owned_value() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_front(world.clone());
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&world));
        assert_eq!(list.back(), Some(&world));
        list.push_front(hello.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn push_back() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&hello));
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn modify_front() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        list.push_front("hello".to_string());
        let world = "world".to_string();
        *list.front_mut().unwrap() = world.clone();
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn clear() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        list.push_back(12u32);
        list.push_back(6u32);
        assert_eq!(list.size(), 2);
        list.clear();
        assert!(list.empty());
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
    }

    struct Test<'a> {
        r: &'a mut u32,
    }

    impl<'a> Drop for Test<'a> {
        fn drop(&mut self) {
            *self.r *= 2;
        }
    }

    #[test]
    fn drop() {
        let mut foo = 1;
        let mut bar = 1;
        {
            moveit! {
                let mut list = unsafe { DefaultList::new() };
            }
            list.push_back(Test { r: &mut foo });
            list.push_back(Test { r: &mut bar });
        }
        assert_eq!(foo, 2);
        assert_eq!(bar, 2);
    }

    #[test]
    fn iter() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        let mut iter = list.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        list.push_back(12u32);
        list.push_back(6u32);
        let iter = list.iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        let vec = iter.collect::<Vec<&u32>>();
        assert_eq!(vec, vec![&12u32, &6u32]);
    }
    #[test]
    fn iter_mut() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        list.push_back(12u32);
        list.push_back(6u32);
        let mut iter = list.iter_mut();
        let first_val = iter.next().unwrap();
        assert_eq!(first_val, &mut 12u32);
        *first_val = 14u32;
        let mut iter = list.iter_mut();
        let first_val = iter.next().unwrap();
        assert_eq!(first_val, &mut 14u32);
        let last_val = iter.last().unwrap();
        assert_eq!(last_val, &mut 6u32);
    }

    #[test]
    fn pop_front() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.pop_front(), Some(hello));
        assert_eq!(list.size(), 1);
        assert_eq!(list.pop_front(), Some(world));
        assert!(list.empty());
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn pop_back() {
        moveit! {
            let mut list = unsafe { DefaultList::new() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.pop_back(), Some(world));
        assert_eq!(list.size(), 1);
        assert_eq!(list.pop_back(), Some(hello));
        assert!(list.empty());
        assert_eq!(list.pop_front(), None);
    }
}
