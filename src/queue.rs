use crate::allocator::{Allocator, DefaultAllocator};
use crate::deque::iter::{Iter, IterMut};
use crate::deque::Deque;
use std::fmt::{Debug, Formatter};

/// Queue with the default allocator.
pub type DefaultQueue<'a, V> = Queue<'a, V, DefaultAllocator>;

/// A first-in, first-out data structure backed by a Deque
#[repr(C)]
pub struct Queue<'a, T: 'a, A: Allocator> {
    deque: Deque<'a, T, A>,
}

unsafe impl<'a, T: Send + 'a, A: Allocator + Send> Send for Queue<'a, T, A> {}
unsafe impl<'a, T: Sync + 'a, A: Allocator + Sync> Sync for Queue<'a, T, A> {}

impl<'a, T: 'a, A: Allocator + Default> Queue<'a, T, A> {
    /// Creates a new empty queue
    fn new() -> Self {
        Self {
            deque: Deque::new(),
        }
    }
}

impl<'a, T: 'a, A: Allocator> Queue<'a, T, A> {
    /// Turns the `Queue` into its inner `Deque`
    pub fn into_inner(self) -> Deque<'a, T, A> {
        self.deque
    }

    /// Returns true if the queue contains no elements
    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    /// Produces an iterator over all of the elements in the queue
    pub fn iter(&self) -> Iter<'a, T> {
        self.deque.iter()
    }

    /// Produces a mutable iterator over all of the elements in the queue
    pub fn iter_mut(&mut self) -> IterMut<'a, T> {
        self.deque.iter_mut()
    }

    /// Returns the number of elements in the queue
    pub fn len(&self) -> usize {
        self.deque.len()
    }

    /// Creates a new queue inside an allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    ///
    /// # Safety
    ///
    /// The allocator specified must safely allocate ande de-allocate valid memory
    pub unsafe fn new_in(allocator: A) -> Self {
        Self {
            deque: Deque::new_in(allocator),
        }
    }

    /// Pops an element from the queue, returning the element on top if there was one
    pub fn pop(&mut self) -> Option<T> {
        self.deque.pop_front()
    }

    /// Pushes an element to the queue
    pub fn push(&mut self, elem: T) {
        self.deque.push_back(elem);
    }

    /// Peeks the top element in the queue without popping it
    pub fn top(&self) -> Option<&T> {
        self.deque.front()
    }
}

impl<'a, T: 'a + Debug, A: Allocator> Debug for Queue<'a, T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deque.fmt(f)
    }
}

impl<'a, T: 'a, A: Allocator + Default> Default for Queue<'a, T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: 'a, A: Allocator + Default> FromIterator<T> for Queue<'a, T, A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            deque: Deque::from_iter(iter),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::queue::DefaultQueue;

    #[test]
    fn layout() {
        assert_eq!(
            std::mem::size_of::<DefaultQueue<u32>>(),
            std::mem::size_of::<usize>() * 11
        );
    }

    #[test]
    fn push_pop() {
        let mut q = DefaultQueue::new();

        assert!(q.is_empty());
        assert_eq!(q.len(), 0);

        for i in 0..256 {
            q.push(i);
        }
        assert_eq!(q.top(), Some(&0));
        assert!(!q.is_empty());
        assert_eq!(q.len(), 256);

        for i in 0..256 {
            assert_eq!(q.pop(), Some(i));
        }

        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn iter() {
        let q: DefaultQueue<i32> = (0..256).collect();

        q.iter().zip(0..256).for_each(|(l, r)| assert_eq!(*l, r));

        let v: Vec<i32> = q.into_iter().collect();

        assert_eq!(v.len(), 256);

        v.iter().zip(0..256).for_each(|(l, r)| assert_eq!(*l, r));
    }
}
