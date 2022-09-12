use std::{
    cmp::min,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::allocator::{Allocator, DefaultAllocator};

/// `Vector` is synonymous to `Vec`, a dynamically-resizing array.
/// The EASTL implementation consists of begin, end, and capacity pointers,
/// as well as a following allocator
#[repr(C)]
pub struct Vector<T: Sized, A: Allocator = DefaultAllocator> {
    /// We've chosen `*mut T` over `NonNull<T>` at the expense of
    /// covariance because EASTL would try to de-allocate a non-null
    /// `begin`, even if it is size zero
    begin_ptr: *mut T,
    end_ptr: *mut T,
    capacity_ptr: *mut T,
    allocator: A,
    _holds_data: PhantomData<T>,
}

impl<T: Sized> Vector<T, DefaultAllocator> {
    /// Creates a new vector
    pub fn new() -> Self {
        unsafe { Self::new_in(DefaultAllocator::default()) }
    }

    /// Creates a new vector with a capacity allocated
    ///
    /// # Arguments
    ///
    /// `capacity`: The initial capacity of the vector
    pub fn with_capacity(capacity: usize) -> Self {
        let mut v = Vector::new();
        v.reserve(capacity);
        v
    }
}

impl<T: Sized, A: Allocator> Vector<T, A> {
    /// Creates a vector from a buffer with a custom allocator
    ///
    /// # Arguments
    ///
    /// `buf`: The buffer
    ///
    /// `allocator`: The allocator used to allocate and de-allocate elements
    ///
    /// # Safety
    ///
    /// The allocator specified must safely allocate ande de-allocate valid memory
    pub unsafe fn from_in(buf: &[T], allocator: A) -> Self {
        let mut this = Self::new_in(allocator);
        this.assign(buf);
        this
    }

    /// Creates a vector with a custom allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator used to allocate and de-allocate elements
    ///
    /// # Safety
    ///
    /// The allocator specified must safely allocate ande de-allocate valid memory
    pub unsafe fn new_in(allocator: A) -> Self {
        Self {
            begin_ptr: std::ptr::null_mut(),
            end_ptr: std::ptr::null_mut(),
            capacity_ptr: std::ptr::null_mut(),
            allocator,
            _holds_data: PhantomData::default(),
        }
    }

    /// Assigns a vector to a slice
    ///
    /// # Arguments
    ///
    /// `buf`: The slice
    pub fn assign(&mut self, buf: &[T]) {
        self.reserve(buf.len());
        unsafe {
            self.begin_ptr.copy_from(buf.as_ptr(), buf.len());
            self.end_ptr = self.begin_ptr.add(buf.len());
        }
    }

    /// Returns the vector as raw bytes
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.begin_ptr, self.len()) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.begin_ptr, self.len()) }
    }

    /// Returns the capacity of the vector
    pub fn capacity(&self) -> usize {
        (unsafe { self.capacity_ptr.offset_from(self.begin_ptr) }) as usize
    }

    /// Returns true if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the array is full to the capacity
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Returns the length of the vector
    pub fn len(&self) -> usize {
        (unsafe { self.end_ptr.offset_from(self.begin_ptr) }) as usize
    }

    /// Pushes a new element into the vector
    ///
    /// # Arguments
    ///
    /// `elem`: The new element
    pub fn push(&mut self, elem: T) {
        // see if we should expand
        if self.is_full() {
            self.grow();
        }
        // add the new element and increment size
        unsafe {
            self.end_ptr.write(elem);
            self.increment_size();
        }
    }

    /// Pops an element off of the back of the array
    pub fn pop(&mut self) -> Option<T> {
        // see if we have any elements to pop
        if self.is_empty() {
            None
        } else {
            unsafe {
                self.decrement_size();
                Some(self.end_ptr.read())
            }
        }
    }

    /// Inserts an element into the array at an index.
    /// `index` must be less than or equal to `size`
    ///
    /// # Arguments
    ///
    /// `index`: The index to insert the element
    ///
    /// `elem`: The element to add to the array
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len(), "index out of bounds");
        // see if we need to expand the array
        if self.is_full() {
            self.grow();
        }
        // shift all elements to the right
        unsafe {
            self.begin_ptr
                .add(index)
                .copy_to(self.begin_ptr.add(index + 1), self.len() - index);
            // move the element
            self.begin_ptr.add(index).write(elem);
            self.increment_size();
        }
    }

    /// Remove the element at the index and return it
    ///
    /// # Arguments
    ///
    /// `index`: The index of the element to remove
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            unsafe {
                // first, read the element
                let res = self.begin_ptr.add(index).read();
                self.decrement_size();
                // shift elements left
                self.begin_ptr
                    .add(index)
                    .copy_from(self.begin_ptr.add(index + 1), self.len() - index);
                Some(res)
            }
        }
    }

    /// Reserves space for elements within the vector
    ///
    /// # Arguments
    ///
    /// `capacity`: The new capacity of the vector
    pub fn reserve(&mut self, capacity: usize) {
        // allocate a new bit of memory
        let size = self.len();
        // allocate the new buffer
        let new_begin_ptr = unsafe { self.allocator.allocate::<T>(capacity) };
        // copy from the old array if we should
        if size > 0 {
            unsafe {
                new_begin_ptr.copy_from(self.begin_ptr, min(size, capacity));
            }
        }
        // calculate and store new pointers
        self.begin_ptr = new_begin_ptr;
        self.end_ptr = unsafe { new_begin_ptr.add(size) };
        self.capacity_ptr = unsafe { new_begin_ptr.add(capacity) }
    }

    /// Incremement the array size
    unsafe fn decrement_size(&mut self) {
        self.end_ptr = self.end_ptr.sub(1);
    }

    /// Decrement the array size
    unsafe fn increment_size(&mut self) {
        self.end_ptr = self.end_ptr.add(1);
    }

    /// Calculates the growing array capacity given its old capacity
    ///
    /// # Arguments
    ///
    /// `old_capacity`: The previous capacity of the array
    fn calculate_grow_capacity(old_capacity: usize) -> usize {
        if old_capacity == 0 {
            1
        } else {
            old_capacity * 2
        }
    }

    /// Grows the array to fit additional elements
    fn grow(&mut self) {
        // allocate a new bit of memory
        let new_capacity = Self::calculate_grow_capacity(self.capacity());
        self.reserve(new_capacity);
    }
}

impl<T, A: Allocator> AsRef<[T]> for Vector<T, A> {
    fn as_ref(&self) -> &[T] {
        &*self
    }
}

impl<T: Debug, A: Allocator> Debug for Vector<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.as_ref()))
    }
}

impl<T, A> Drop for Vector<T, A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        if !self.begin_ptr.is_null() {
            unsafe {
                // drop all elements in place
                std::ptr::drop_in_place(self.as_slice_mut());
                // free the array
                self.allocator.deallocate::<T>(self.begin_ptr, self.len())
            }
        }
    }
}

impl<T> Default for Vector<T, DefaultAllocator> {
    fn default() -> Self {
        Vector::new()
    }
}

impl<T, A: Allocator> Deref for Vector<T, A> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, A: Allocator> DerefMut for Vector<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.begin_ptr, self.len()) }
    }
}

impl<T: Sized> From<&[T]> for Vector<T, DefaultAllocator> {
    fn from(buf: &[T]) -> Self {
        let mut v = Vector::new();
        v.assign(buf);
        v
    }
}

impl<T: Sized> From<&mut [T]> for Vector<T, DefaultAllocator> {
    fn from(buf: &mut [T]) -> Self {
        let mut v = Vector::new();
        v.assign(buf);
        v
    }
}

impl<T: Sized, const N: usize> From<[T; N]> for Vector<T, DefaultAllocator> {
    fn from(buf: [T; N]) -> Self {
        let mut v = Vector::with_capacity(buf.len());
        // move all values in
        for val in buf {
            v.push(val);
        }
        v
    }
}

impl<T: Sized, const N: usize> From<&[T; N]> for Vector<T, DefaultAllocator> {
    fn from(buf: &[T; N]) -> Self {
        let mut v = Vector::new();
        v.assign(buf);
        v
    }
}

impl<T> FromIterator<T> for Vector<T, DefaultAllocator> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        let mut v = Vector::with_capacity(lower_bound);
        for elem in iter {
            v.push(elem);
        }
        v
    }
}

unsafe impl<T: Send, A: Allocator + Send> Send for Vector<T, A> {}
unsafe impl<T: Sync, A: Allocator + Sync> Sync for Vector<T, A> {}

#[cfg(test)]
mod test {
    use memoffset::offset_of;

    use super::Vector;

    #[test]
    fn layout() {
        assert_eq!(offset_of!(Vector<u32>, begin_ptr), 0);
        assert_eq!(
            offset_of!(Vector<u32>, end_ptr),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(Vector<u32>, capacity_ptr),
            std::mem::size_of::<usize>() * 2
        );
        assert_eq!(
            offset_of!(Vector<u32>, allocator),
            std::mem::size_of::<usize>() * 3
        );
        assert_eq!(
            std::mem::size_of::<Vector<u32>>(),
            std::mem::size_of::<usize>() * 4
        );
    }

    #[test]
    fn empty_vec() {
        let v: Vector<u32> = Vector::new();
        assert!(v.begin_ptr.is_null());
        assert!(v.end_ptr.is_null());
        assert!(v.capacity_ptr.is_null());
        assert_eq!(v.capacity(), 0);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
    }

    #[test]
    fn push_one() {
        let mut v = Vector::new();
        v.push(20);
        assert_eq!(v.capacity(), 1);
        assert_eq!(v.len(), 1);
        assert!(!v.is_empty());
        let arr = &*v;
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], 20);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.capacity(), 1);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
    }

    #[test]
    fn push_two() {
        let mut v = Vector::new();
        v.push(20);
        v.push(25);
        assert_eq!(v.capacity(), 2);
        assert_eq!(v.len(), 2);
        assert!(!v.is_empty());
        let arr = &mut *v;
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], 20);
        arr[1] = 30;
        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.capacity(), 2);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
    }

    #[test]
    fn push_three() {
        let mut v = Vector::new();
        v.push(20);
        v.push(25);
        v.push(30);
        assert_eq!(v.capacity(), 4);
        assert_eq!(v.len(), 3);
        assert!(!v.is_empty());
        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.len(), 2);
        assert_eq!(v.pop(), Some(25));
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.capacity(), 4);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
    }

    #[test]
    fn insert() {
        let mut v = Vector::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        v.insert(2, 5);
        assert_eq!(&*v, &[1, 2, 5, 3, 4]);
        assert_eq!(v.capacity(), 8);
    }

    #[test]
    fn remove() {
        let mut v = Vector::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        assert_eq!(v.remove(2), Some(3));
        assert_eq!(&*v, &[1, 2, 4]);
    }

    #[test]
    fn iter() {
        let mut v = Vector::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.iter().sum::<i32>(), 6);
    }

    #[test]
    fn from() {
        let v = Vector::from(&[1, 2, 3]);
        assert_eq!(v.capacity(), 3);
        assert_eq!(v.len(), 3);
        assert_eq!(&*v, &[1, 2, 3]);
    }

    #[test]
    fn from_iter() {
        let v = (1..4).collect::<Vector<_>>();
        assert_eq!(v.capacity(), 3);
        assert_eq!(v.len(), 3);
        assert_eq!(&*v, &[1, 2, 3]);
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
            let _ = Vector::from([Test { r: &mut foo }, Test { r: &mut bar }]);
        }
        assert_eq!(foo, 2);
        assert_eq!(bar, 2);
    }
}
