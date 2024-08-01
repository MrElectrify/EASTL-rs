use crate::allocator::{Allocator, DefaultAllocator};
use crate::deque::iter::{CompatIterMut, Iter, IterMut};
use crate::util::rotate;
use itertools::Itertools;
use std::fmt::{Debug, Formatter};

pub mod iter;

/// Deque with the default allocator.
pub type DefaultDeque<'a, V> = Deque<'a, V, DefaultAllocator>;

/// A double-ended queue implemented with multiple arrays
#[repr(C)]
pub struct Deque<'a, T: 'a, A: Allocator> {
    ptr_array: *mut *mut T,
    ptr_array_size: u32,
    begin_it: CompatIterMut<'a, T>,
    end_it: CompatIterMut<'a, T>,
    allocator: A,
}

unsafe impl<'a, T: Send + 'a, A: Allocator + Send> Send for Deque<'a, T, A> {}
unsafe impl<'a, T: Sync + 'a, A: Allocator + Sync> Sync for Deque<'a, T, A> {}

impl<'a, T: 'a, A: Allocator + Default> Deque<'a, T, A> {
    /// Creates a new deque in the default allocator
    pub fn new() -> Self {
        unsafe { Self::new_in(A::default()) }
    }
}

impl<'a, T: 'a, A: Allocator> Deque<'a, T, A> {
    const INITIAL_PTR_ARRAY_SIZE: u32 = 8;
    const SUBARRAY_SIZE: usize = Self::calculate_subarray_size();

    /// Provides a reference to the back element, or `None` if the deque is empty.
    pub fn back(&self) -> Option<&T> {
        self.iter().next_back()
    }

    /// Provides a mutable reference to the back element, or `None` if the deque is empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.iter_mut().next_back()
    }

    /// Provides a reference to the front element, or `None` if the deque is empty.
    pub fn front(&self) -> Option<&T> {
        self.iter().next()
    }

    /// Provides a mutable reference to the front element, or `None` if the deque is empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.iter_mut().next()
    }

    /// Returns true if the deque contains no elements
    pub fn is_empty(&self) -> bool {
        self.begin_it.current == self.end_it.current
    }

    /// Returns an iterator over the deque
    pub fn iter(&self) -> Iter<'a, T> {
        unsafe { Iter::from_compat((&self.begin_it).into(), (&self.end_it).into()) }
    }

    /// Returns a mutable iterator over the deque
    pub fn iter_mut(&mut self) -> IterMut<'a, T> {
        unsafe { IterMut::from_compat(self.begin_it.clone(), self.end_it.clone()) }
    }

    /// Returns the number of elements in the deque.
    pub fn len(&self) -> usize {
        if self.begin_it.current_array == self.end_it.current_array {
            // super simple calculation
            unsafe { self.end_it.current.offset_from(self.begin_it.current) as usize }
        } else {
            // not so simple...
            let full_subarray_diff = unsafe {
                self.end_it
                    .current_array
                    .offset_from(self.begin_it.current_array)
            } * Self::SUBARRAY_SIZE as isize;
            let begin_subarray_offset =
                unsafe { self.begin_it.current.offset_from(self.begin_it.begin) };
            let end_subarray_offset = unsafe { self.end_it.current.offset_from(self.end_it.begin) };
            // this handles the wrapping
            let subarray_diff = end_subarray_offset - begin_subarray_offset;
            (full_subarray_diff + subarray_diff) as usize
        }
    }

    /// Creates a new deque inside an allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    ///
    /// # Safety
    ///
    /// The allocator specified must safely allocate ande de-allocate valid memory
    pub unsafe fn new_in(allocator: A) -> Self {
        let mut this = Self {
            ptr_array: std::ptr::null_mut(),
            ptr_array_size: 0,
            begin_it: CompatIterMut::default(),
            end_it: CompatIterMut::default(),
            allocator,
        };
        this.init();
        this
    }

    /// Removes the last element from the deque and returns it, or `None` if it is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else if self.end_it.current != self.end_it.begin {
            // no de-allocation necessary!
            self.end_it.current = unsafe { self.end_it.current.sub(1) };

            Some(unsafe { self.end_it.current.read() })
        } else {
            // we need to de-allocate the current array pointer
            self.free_subarray(self.end_it.begin);

            // setup the end iterator again
            unsafe {
                self.end_it
                    .set_subarray(self.end_it.current_array.sub(1), Self::SUBARRAY_SIZE);
                self.end_it.current = self.end_it.end.sub(1);
            };

            // simply read the last value in the subarray
            Some(unsafe { self.end_it.current.read() })
        }
    }

    /// Removes the first element from the deque and returns it, or `None` if it is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else if self.begin_it.current != unsafe { self.begin_it.end.sub(1) } {
            // no de-allocation necessary!
            let elem = unsafe { self.begin_it.current.read() };
            self.begin_it.current = unsafe { self.begin_it.current.add(1) };

            Some(elem)
        } else {
            // read the element before deallocation
            let elem = unsafe { self.begin_it.current.read() };

            // we need to de-allocate the current array pointer
            self.free_subarray(self.begin_it.begin);

            // setup the begin iterator again
            unsafe {
                self.begin_it
                    .set_subarray(self.begin_it.current_array.add(1), Self::SUBARRAY_SIZE);
                self.begin_it.current = self.begin_it.begin;
            }

            Some(elem)
        }
    }

    /// Pushes an element to the back of the deque
    ///
    /// # Arguments
    ///
    /// `elem`: The element
    pub fn push_back(&mut self, elem: T) {
        if self.end_it.current != unsafe { self.end_it.end.sub(1) } {
            // simply add the element to the back of the current subarray
            unsafe {
                self.end_it.current.write(elem);
                self.end_it.current = self.end_it.current.add(1);
            }
        } else {
            // see if we need to allocate more pointers
            if self.end_it.current_array
                == unsafe { self.ptr_array.add(self.ptr_array_size as usize - 1) }
            {
                self.realloc_ptr_array(1, false);
            }
            // write our element to the last position in the subarray
            unsafe { self.end_it.current.write(elem) };
            // allocate a new subarray
            unsafe {
                *self.end_it.current_array.add(1) = self.allocate_subarray();
                self.end_it
                    .set_subarray(self.end_it.current_array.add(1), Self::SUBARRAY_SIZE);
                self.end_it.current = self.end_it.begin;
            };
        }
    }

    /// Pushes an element to the front of the deque
    ///
    /// # Arguments
    ///
    /// `elem`: The element
    pub fn push_front(&mut self, elem: T) {
        if self.begin_it.current != self.begin_it.begin {
            // simply add the element to the front of the current occupied subarray
            unsafe {
                self.begin_it.current = self.begin_it.current.sub(1);
            }
        } else {
            // see if we need to allocate more pointers
            if self.begin_it.current_array == self.ptr_array {
                self.realloc_ptr_array(1, true);
            }

            // allocate a new subarray
            unsafe {
                *self.begin_it.current_array.sub(1) = self.allocate_subarray();
                self.begin_it
                    .set_subarray(self.begin_it.current_array.sub(1), Self::SUBARRAY_SIZE);
                self.begin_it.current = self.begin_it.end.sub(1);
            };
        }
        // write the element
        unsafe {
            self.begin_it.current.write(elem);
        }
    }

    /// Removes and returns the element at `index` from the deque.
    /// Whichever end is closer to the removal point will be moved to make
    /// room, and all the affected elements will be moved to new positions.
    /// Returns `None` if `index` is out of bounds.
    ///
    /// Element at index 0 is the front of the queue.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }

        if index < (len / 2) {
            // we want to copy elements forward
            let elem_it = unsafe { self.iter_mut_unchecked() }
                .rev()
                .skip(len - index - 1);
            let next_it = unsafe { self.iter_mut_unchecked() }.rev().skip(len - index);
            unsafe { rotate(elem_it, next_it) };
            self.pop_front()
        } else {
            // we want to copy elements backward
            let elem_it = unsafe { self.iter_mut_unchecked() }.skip(index);
            let next_it = unsafe { self.iter_mut_unchecked() }.skip(index + 1);
            unsafe { rotate(elem_it, next_it) };
            self.pop_back()
        }
    }

    /// Allocates the subarray pointer array
    ///
    /// # Arguments
    ///
    /// `number_of_ptrs`: The number of pointers to allocate
    fn allocate_ptr_array<'b>(&mut self, number_of_ptrs: usize) -> &'b mut [*mut T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.allocator.allocate::<*mut T>(number_of_ptrs),
                number_of_ptrs,
            )
        }
    }

    /// Allocates a subarray
    fn allocate_subarray(&mut self) -> *mut T {
        self.allocator.allocate::<T>(Self::SUBARRAY_SIZE)
    }

    /// Frees the subarray pointer array
    fn free_ptr_array(&mut self) {
        unsafe {
            self.allocator
                .deallocate(self.ptr_array, self.ptr_array_size as usize)
        }
    }

    /// Frees a subarray
    ///
    /// # Arguments
    ///
    /// `subarray`: The subarray to free
    fn free_subarray(&mut self, subarray: *mut T) {
        unsafe { self.allocator.deallocate(subarray, Self::SUBARRAY_SIZE) }
    }

    /// Calculates the size of each sub-array
    const fn calculate_subarray_size() -> usize {
        let elem_size = std::mem::size_of::<T>();
        if elem_size <= 4 {
            64
        } else if elem_size <= 8 {
            32
        } else if elem_size <= 16 {
            16
        } else if elem_size <= 32 {
            8
        } else {
            4
        }
    }

    /// Initializes the subarray
    fn init(&mut self) {
        self.ptr_array_size = Self::INITIAL_PTR_ARRAY_SIZE;

        // allocate the subarrays
        let ptr_array = self.allocate_ptr_array(self.ptr_array_size as usize);
        ptr_array.fill_with(std::ptr::null_mut);

        // we start with an empty deque, so only allocate the first array mid-way through
        ptr_array[((Self::INITIAL_PTR_ARRAY_SIZE - 1) / 2) as usize] = self.allocate_subarray();

        // setup the iterators
        unsafe {
            self.begin_it
                .set_subarray(&mut ptr_array[3], Self::SUBARRAY_SIZE);
            self.begin_it.current = self.begin_it.begin;
            self.end_it
                .set_subarray(&mut ptr_array[3], Self::SUBARRAY_SIZE);
            self.end_it.current = self.end_it.begin;
        };

        self.ptr_array = ptr_array.as_mut_ptr();
    }

    /// Returns a mutable iterator over the deque.
    ///
    /// # Safety
    /// Elides the borrow checker.
    unsafe fn iter_mut_unchecked(&mut self) -> IterMut<'a, T> {
        IterMut::from_compat(self.begin_it.clone(), self.end_it.clone())
    }

    /// Reallocates the pointer array with additional capacity
    ///
    /// # Arguments
    ///
    /// `additional_capacity`: The additional needed capacity
    ///
    /// `front`: True if the reallocation is needed on the front of the array
    fn realloc_ptr_array(&mut self, mut additional_capacity: usize, front: bool) {
        let unused_ptrs_at_front =
            unsafe { self.begin_it.current_array.offset_from(self.ptr_array) } as usize;
        let used_ptrs = (unsafe {
            self.end_it
                .current_array
                .offset_from(self.begin_it.current_array)
        } + 1) as usize;
        let unused_ptrs_at_back = (self.ptr_array_size as usize - unused_ptrs_at_front) - used_ptrs;
        let current_array_start =
            unsafe { self.begin_it.current_array.offset_from(self.ptr_array) } as usize;
        let current_array_end = current_array_start + used_ptrs;
        let ptr_array = if let Some(ptr_array) = unsafe { self.ptr_array.as_mut() } {
            unsafe { std::slice::from_raw_parts_mut(ptr_array, self.ptr_array_size as usize) }
        } else {
            &mut []
        };

        let new_array_start;

        // see if we can make use of extra space without re-allocating
        if !front && additional_capacity <= unused_ptrs_at_front {
            // if there's a lot of extra space then they are likely using the deque with heavy use
            // of `push_back` (like `queue`), so allocate them even more space. the + 1 is a
            // difference to the official implementation to hit this optimization earlier
            if additional_capacity < (unused_ptrs_at_front + 1) / 2 {
                additional_capacity = (unused_ptrs_at_front + 1) / 2;
            }

            new_array_start = unused_ptrs_at_front - additional_capacity;

            ptr_array.copy_within(current_array_start..current_array_end, new_array_start);
        } else if front && additional_capacity <= unused_ptrs_at_back {
            // if there's a lot of extra space then they are likely using the deque with heavy use
            // of `push_front`, so allocate them even more space
            if additional_capacity < unused_ptrs_at_back / 2 {
                additional_capacity = unused_ptrs_at_back / 2;
            }

            new_array_start = current_array_start + additional_capacity;

            // move the pointers within. note that this will leave the old pointers behind, but we
            // are using iterators to track that so it's fine
            ptr_array.copy_within(current_array_start..current_array_end, new_array_start);
        } else {
            let new_ptr_array_size =
                self.ptr_array_size + self.ptr_array_size.max(additional_capacity as u32) + 2;
            // allocate at least double + 2 pointers
            let new_ptr_array = self.allocate_ptr_array(new_ptr_array_size as usize);

            // copy the old pointers over
            new_array_start = unused_ptrs_at_front + if front { additional_capacity } else { 0 };
            let new_array_end = new_array_start + used_ptrs;
            new_ptr_array[new_array_start..new_array_end]
                .copy_from_slice(&ptr_array[current_array_start..current_array_end]);

            self.free_ptr_array();

            self.ptr_array = new_ptr_array.as_mut_ptr();
            self.ptr_array_size = new_ptr_array_size;
        }

        // update the iterators
        unsafe {
            self.begin_it
                .set_subarray(self.ptr_array.add(new_array_start), Self::SUBARRAY_SIZE)
        };
        unsafe {
            self.end_it.set_subarray(
                self.ptr_array.add((new_array_start + used_ptrs) - 1),
                Self::SUBARRAY_SIZE,
            )
        };
    }
}

impl<'a, T: 'a + Debug, A: Allocator> Debug for Deque<'a, T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ {:?} ]", self.iter().format(", "))
    }
}

impl<'a, T: 'a, A: Allocator + Default> Default for Deque<'a, T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: 'a, A: Allocator> Drop for Deque<'a, T, A> {
    fn drop(&mut self) {
        // drop all elements
        for elem in self.iter_mut() {
            unsafe { std::ptr::drop_in_place(elem as *mut T) }
        }

        // free the sub-arrays
        if let Some(current_array) = unsafe { self.begin_it.current_array.as_mut() } {
            for subarray in unsafe {
                std::slice::from_raw_parts_mut(
                    current_array,
                    self.end_it.current_array.offset_from(current_array) as usize,
                )
            } {
                self.free_subarray(*subarray);
            }
        }

        self.free_ptr_array();
    }
}

impl<'a, T: 'a, A: Allocator + Default> FromIterator<T> for Deque<'a, T, A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut d = Self::new();
        iter.into_iter().for_each(|elem| d.push_back(elem));
        d
    }
}

#[cfg(test)]
mod test {
    use crate::deque::DefaultDeque;
    use memoffset::offset_of;

    #[test]
    fn layout() {
        assert_eq!(offset_of!(DefaultDeque::<u32>, ptr_array), 0);
        assert_eq!(
            offset_of!(DefaultDeque::<u32>, ptr_array_size),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(DefaultDeque::<u32>, begin_it),
            std::mem::size_of::<usize>() * 2
        );
        assert_eq!(
            offset_of!(DefaultDeque::<u32>, end_it),
            std::mem::size_of::<usize>() * 6
        );
        assert_eq!(
            offset_of!(DefaultDeque::<u32>, allocator),
            std::mem::size_of::<usize>() * 10
        );
        assert_eq!(
            std::mem::size_of::<DefaultDeque<u32>>(),
            std::mem::size_of::<usize>() * 11
        )
    }

    #[test]
    fn initial_state() {
        let d = DefaultDeque::<u32>::default();
        assert!(!d.ptr_array.is_null());
        assert_eq!(d.ptr_array_size, 8);
        assert_eq!(d.begin_it.begin, unsafe { *d.ptr_array.add(3) });
        assert_eq!(d.begin_it.begin, d.begin_it.current);
        assert_eq!(d.end_it.begin, d.begin_it.begin);
        assert_eq!(d.end_it.begin, d.begin_it.current);
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn push_front() {
        let mut d = DefaultDeque::new();

        d.push_front(0);
        assert!(!d.is_empty());
        assert_eq!(d.len(), 1);

        d.push_front(1);
        assert!(!d.is_empty());
        assert_eq!(d.len(), 2);

        for i in 2..65 {
            d.push_front(i);
        }
        assert!(!d.is_empty());
        assert_eq!(d.len(), 65);
    }

    #[test]
    fn push_back() {
        let mut d = DefaultDeque::new();

        d.push_back(0);
        assert!(!d.is_empty());
        assert_eq!(d.len(), 1);

        d.push_back(0);
        assert!(!d.is_empty());
        assert_eq!(d.len(), 2);

        for i in 2..65 {
            d.push_back(i);
        }
        assert!(!d.is_empty());
        assert_eq!(d.len(), 65);
    }

    #[test]
    fn push_front_and_back() {
        let mut d = DefaultDeque::new();

        d.push_front(0);
        d.push_back(1);
        assert!(!d.is_empty());
        assert_eq!(d.len(), 2);

        for i in 2..65 {
            d.push_front(i);
            d.push_back(i);
        }
        assert!(!d.is_empty());
        assert_eq!(d.len(), 128);
    }

    #[test]
    fn from_iter() {
        let d: DefaultDeque<u32> = (0..10).collect();

        assert!(!d.is_empty());
        assert_eq!(d.len(), 10);
    }

    #[test]
    fn front() {
        let mut d = DefaultDeque::new();
        assert_eq!(d.front(), None);

        d.push_front(1);
        assert_eq!(d.front(), Some(&1));

        d.push_front(2);
        assert_eq!(d.front(), Some(&2));

        d.push_back(3);
        assert_eq!(d.front(), Some(&2));
    }

    #[test]
    fn back() {
        let mut d = DefaultDeque::new();
        assert_eq!(d.back(), None);

        d.push_back(1);
        assert_eq!(d.back(), Some(&1));

        d.push_back(2);
        assert_eq!(d.back(), Some(&2));

        d.push_front(3);
        assert_eq!(d.back(), Some(&2));
    }

    #[test]
    fn realloc_ptr_back() {
        let mut d = DefaultDeque::new();

        for i in 0..512 {
            d.push_back(i);
        }

        for i in (0..512).rev() {
            assert_eq!(d.pop_back(), Some(i));
        }

        assert_eq!(d.pop_back(), None);
        assert_eq!(d.pop_front(), None);

        assert!(d.is_empty());
    }

    #[test]
    fn realloc_ptr_front() {
        let mut d = DefaultDeque::new();

        for i in 0..512 {
            d.push_front(i);
        }

        for i in (0..512).rev() {
            assert_eq!(d.pop_front(), Some(i));
        }

        assert_eq!(d.pop_back(), None);
        assert_eq!(d.pop_front(), None);

        assert!(d.is_empty());
    }

    #[test]
    fn remove_out_of_bounds() {
        let mut d = (0..6).collect::<DefaultDeque<_>>();

        assert!(d.remove(6).is_none());

        itertools::assert_equal(d, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn remove_front() {
        let mut d = (0..6).collect::<DefaultDeque<_>>();

        assert_eq!(d.remove(0), Some(0));

        itertools::assert_equal(d, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn remove_back() {
        let mut d = (0..6).collect::<DefaultDeque<_>>();

        assert_eq!(d.remove(5), Some(5));

        itertools::assert_equal(d, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn remove_middle_front_half() {
        let mut d = (0..6).collect::<DefaultDeque<_>>();

        assert_eq!(d.remove(1), Some(1));

        itertools::assert_equal(d, vec![0, 2, 3, 4, 5]);
    }

    #[test]
    fn remove_middle_back_half() {
        let mut d = (0..6).collect::<DefaultDeque<_>>();

        d.remove(4);

        itertools::assert_equal(d, vec![0, 1, 2, 3, 5]);
    }
}
