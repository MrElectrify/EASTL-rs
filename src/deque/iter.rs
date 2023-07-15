use crate::allocator::Allocator;
use crate::deque::Deque;
use crate::queue::Queue;
use std::marker::PhantomData;

/// A compatibility array mimicking the C++ one.
/// Most of deque is actually implemented using this
/// iterator anyways, so there's no getting around it
#[repr(C)]
pub struct CompatIter<'a, T: 'a> {
    pub(crate) current: *const T,
    pub(crate) begin: *const T,
    pub(crate) end: *const T,
    pub(crate) current_array: *const *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> From<CompatIterMut<'a, T>> for CompatIter<'a, T> {
    fn from(iter_mut: CompatIterMut<'a, T>) -> Self {
        Self {
            current: iter_mut.current,
            begin: iter_mut.begin,
            end: iter_mut.end,
            current_array: iter_mut.current_array as *const *const T,
            _marker: Default::default(),
        }
    }
}

impl<'a, T: 'a> From<&CompatIterMut<'a, T>> for CompatIter<'a, T> {
    fn from(iter_mut: &CompatIterMut<'a, T>) -> Self {
        Self {
            current: iter_mut.current,
            begin: iter_mut.begin,
            end: iter_mut.end,
            current_array: iter_mut.current_array as *const *const T,
            _marker: Default::default(),
        }
    }
}

/// A compatibility array mimicking the C++ one.
/// Most of deque is actually implemented using this
/// iterator anyways, so there's no getting around it
#[repr(C)]
pub struct CompatIterMut<'a, T: 'a> {
    pub(crate) current: *mut T,
    pub(crate) begin: *mut T,
    pub(crate) end: *mut T,
    pub(crate) current_array: *mut *mut T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> CompatIterMut<'a, T> {
    /// Clones a mutable iterator
    ///
    /// # Safety
    ///
    /// This must not create 2 public mutable iterators
    pub(crate) unsafe fn clone(&self) -> Self {
        Self {
            current: self.current,
            begin: self.begin,
            end: self.end,
            current_array: self.current_array,
            _marker: Default::default(),
        }
    }

    /// Sets the subarray for an iterator
    ///
    /// # Arguments
    ///
    /// `subarray`: The subarray
    ///
    /// `subarray_size`: The size of the subarray
    ///
    /// # Safety
    ///
    /// `subarray` must point to a valid subarray
    pub(crate) unsafe fn set_subarray(&mut self, subarray: *mut *mut T, subarray_size: usize) {
        self.current_array = subarray;
        self.begin = *subarray;
        self.end = self.begin.add(subarray_size);
    }
}

impl<'a, T: 'a> Default for CompatIterMut<'a, T> {
    fn default() -> Self {
        Self {
            current: std::ptr::null_mut(),
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            current_array: std::ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

/// An iterator of a deque
pub struct RawIter<'a, T: 'a> {
    current: *mut T,
    current_arr: *mut *mut T,
    last: *mut T,
    last_arr: *mut *mut T,
    subarray_size: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Iterator for RawIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.last {
            None
        } else {
            let elem = unsafe { self.current.as_mut() };
            if self.current == unsafe { (*self.current_arr).add(self.subarray_size - 1) } {
                // we reached the end of the current subarray. advance 1
                self.current_arr = unsafe { self.current_arr.add(1) };
                self.current = unsafe { *self.current_arr };
            } else {
                self.current = unsafe { self.current.add(1) }
            }
            elem
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for RawIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.last == self.current {
            None
        } else {
            // first, we need to go back a subarray
            if self.last == unsafe { *self.last_arr } {
                // we reached the beginning of the current subarray. go back 1
                self.last_arr = unsafe { self.last_arr.sub(1) };
                self.last = unsafe { (*self.last_arr).add(self.subarray_size - 1) };
            } else {
                self.last = unsafe { self.last.sub(1) };
            }
            unsafe { self.last.as_mut() }
        }
    }
}

impl<'a, T: 'a> RawIter<'a, T> {
    /// Transforms a raw iterator into the compatible component parts
    fn into_compat(self) -> (CompatIter<'a, T>, CompatIter<'a, T>) {
        (
            CompatIter {
                current: self.current,
                begin: unsafe { *self.current_arr },
                end: unsafe { (*self.current_arr).add(self.subarray_size) },
                current_array: self.current_arr as *const *const T,
                _marker: Default::default(),
            },
            CompatIter {
                current: self.last,
                begin: unsafe { *self.last_arr },
                end: unsafe { (*self.last_arr).add(self.subarray_size) },
                current_array: self.last_arr as *const *const T,
                _marker: Default::default(),
            },
        )
    }

    /// Transforms a raw iterator into the compatible component parts
    ///
    /// # Safety
    ///
    /// Mutable-ness must be preserved
    unsafe fn into_compat_mut(self) -> (CompatIterMut<'a, T>, CompatIterMut<'a, T>) {
        (
            CompatIterMut {
                current: self.current,
                begin: unsafe { *self.current_arr },
                end: unsafe { (*self.current_arr).add(self.subarray_size) },
                current_array: self.current_arr,
                _marker: Default::default(),
            },
            CompatIterMut {
                current: self.last,
                begin: unsafe { *self.last_arr },
                end: unsafe { (*self.last_arr).add(self.subarray_size) },
                current_array: self.last_arr,
                _marker: Default::default(),
            },
        )
    }

    /// Constructs a raw iterator from a pair of mutable compatibility iterators
    ///
    /// # Arguments
    ///
    /// `begin`: The begin iterator
    ///
    /// `end`: The end iterator
    ///
    /// # Safety
    ///
    /// Mutability must be preserved.
    ///
    /// `begin` and `end` must point to valid portions of the deque, and end must be after begin
    unsafe fn from_compat(begin: CompatIter<'a, T>, end: CompatIter<'a, T>) -> Self {
        Self {
            current: begin.current as *mut T,
            current_arr: begin.current_array as *mut *mut T,
            last: end.current as *mut T,
            last_arr: end.current_array as *mut *mut T,
            subarray_size: unsafe { begin.end.offset_from(begin.begin) } as usize,
            _marker: PhantomData,
        }
    }

    /// Constructs a raw iterator from a pair of mutable compatibility iterators
    ///
    /// # Arguments
    ///
    /// `begin`: The begin iterator
    ///
    /// `end`: The end iterator
    ///
    /// # Safety
    ///
    /// `begin` and `end` must point to valid portions of the deque, and end must be after begin
    unsafe fn from_compat_mut(begin: CompatIterMut<'a, T>, end: CompatIterMut<'a, T>) -> Self {
        Self {
            current: begin.current,
            current_arr: begin.current_array,
            last: end.current,
            last_arr: end.current_array,
            subarray_size: unsafe { begin.end.offset_from(begin.begin) } as usize,
            _marker: PhantomData,
        }
    }
}

/// An iterator of a deque
pub struct Iter<'a, T: 'a> {
    raw: RawIter<'a, T>,
}

impl<'a, T: 'a> Iter<'a, T> {
    /// Constructs a Rust iterator from a pair of compatibility iterators
    ///
    /// # Arguments
    ///
    /// `begin`: The begin iterator
    ///
    /// `end`: The end iterator
    ///
    /// # Safety
    ///
    /// `begin` and `end` must point to valid portions of the deque, and end must be after begin
    pub unsafe fn from_compat(begin: CompatIter<'a, T>, end: CompatIter<'a, T>) -> Self {
        Self {
            raw: RawIter::from_compat(begin, end),
        }
    }

    /// Converts the Rust iterator into a set of C++ compatibility iterators
    pub fn into_compat(self) -> (CompatIter<'a, T>, CompatIter<'a, T>) {
        self.raw.into_compat()
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|r| &*r)
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back().map(|r| &*r)
    }
}

/// An iterator of a deque
pub struct IterMut<'a, T: 'a> {
    raw: RawIter<'a, T>,
}

impl<'a, T: 'a> IterMut<'a, T> {
    /// Constructs a Rust iterator from a pair of mutable compatibility iterators
    ///
    /// # Arguments
    ///
    /// `begin`: The begin iterator
    ///
    /// `end`: The end iterator
    ///
    /// # Safety
    ///
    /// `begin` and `end` must point to valid portions of the deque, and end must be after begin
    pub unsafe fn from_compat(begin: CompatIterMut<'a, T>, end: CompatIterMut<'a, T>) -> Self {
        Self {
            raw: RawIter::from_compat_mut(begin, end),
        }
    }

    /// Converts the Rust iterator into a set of C++ compatibility iterators
    pub fn into_compat(self) -> (CompatIter<'a, T>, CompatIter<'a, T>) {
        self.raw.into_compat()
    }

    /// Converts the Rust iterator into a set of C++ compatibility iterators
    pub fn into_compat_mut(self) -> (CompatIterMut<'a, T>, CompatIterMut<'a, T>) {
        unsafe { self.raw.into_compat_mut() }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next()
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back()
    }
}

/// A consuming iterator
pub struct IntoIter<'a, T: 'a, A: Allocator> {
    deque: Deque<'a, T, A>,
}

impl<'a, T: 'a, A: Allocator> Iterator for IntoIter<'a, T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.deque.pop_front()
    }
}

impl<'a, T: 'a, A: Allocator> DoubleEndedIterator for IntoIter<'a, T, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deque.pop_back()
    }
}

impl<'a, T: 'a, A: Allocator> IntoIterator for Deque<'a, T, A> {
    type Item = T;
    type IntoIter = IntoIter<'a, T, A>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { deque: self }
    }
}

impl<'a, T: 'a, A: Allocator> IntoIterator for Queue<'a, T, A> {
    type Item = T;
    type IntoIter = IntoIter<'a, T, A>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            deque: self.into_inner(),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::deque::iter::CompatIterMut;
    use crate::deque::DefaultDeque;
    use memoffset::offset_of;

    #[test]
    fn layout() {
        assert_eq!(offset_of!(CompatIterMut<u32>, current), 0);
        assert_eq!(
            offset_of!(CompatIterMut<u32>, begin),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(CompatIterMut<u32>, end),
            std::mem::size_of::<usize>() * 2
        );
        assert_eq!(
            offset_of!(CompatIterMut<u32>, current_array),
            std::mem::size_of::<usize>() * 3
        );
        assert_eq!(
            std::mem::size_of::<CompatIterMut<u32>>(),
            std::mem::size_of::<usize>() * 4
        );
    }

    #[test]
    fn empty_iter() {
        let d = DefaultDeque::<u32>::new();
        let mut i = d.iter();

        assert_eq!(i.next(), None);
        assert_eq!(i.next_back(), None);
    }

    #[test]
    fn iter() {
        let d: DefaultDeque<u32> = (0..10).collect();
        let mut i = d.iter();

        for elem in 0..5 {
            assert_eq!(i.next(), Some(&elem));
        }
        for elem in (5..10).rev() {
            assert_eq!(i.next_back(), Some(&elem));
        }

        assert_eq!(i.next(), None);
        assert_eq!(i.next_back(), None);
    }

    #[test]
    fn iter_across_borders() {
        let mut d = DefaultDeque::new();

        // make sure front and back have values so we go over boundaries
        for i in 0..70 {
            d.push_front(i);
            d.push_back(i);
        }

        let mut i = d.iter();

        for elem in (0..70).rev() {
            assert_eq!(i.next(), Some(&elem));
        }
        for elem in (0..70).rev() {
            assert_eq!(i.next_back(), Some(&elem));
        }

        assert_eq!(i.next(), None);
        assert_eq!(i.next_back(), None);
    }
}
