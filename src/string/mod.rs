mod null_terminator_allocator;

use std::convert::Infallible;
use std::str::FromStr;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use crate::allocator::DefaultAllocator;
use crate::string::null_terminator_allocator::NullTerminatorAllocator;
use crate::{
    allocator::Allocator,
    hash::{DefaultHash, Hash},
    vector::Vector,
};

/// String with the default allocator;
pub type DefaultString = String<DefaultAllocator>;

/// `String` is what it sounds like, a string of characters.
/// It's actually implemented internally as a vector
pub struct String<A: Allocator> {
    vec: Vector<u8, NullTerminatorAllocator<A>>,
}

impl<A: Allocator + Default> String<A> {
    /// Creates a new empty string
    pub fn new() -> Self {
        Self { vec: Vector::new() }
    }

    /// Creates a new string with a capacity allocated
    ///
    /// # Arguments
    ///
    /// `capacity`: The initial capacity of the string
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            // make space for the null terminator
            vec: Vector::with_capacity(capacity),
        }
    }
}

impl<A: Allocator> String<A> {
    /// Creates a string from a string slice with a custom allocator
    ///
    /// # Arguments
    ///
    /// `s`: The string slice
    ///
    /// `allocator`: The allocator used to allocate and de-allocate elements
    ///
    /// # Safety
    ///
    /// The allocator specified must safely allocate ande de-allocate valid memory
    pub unsafe fn from_in<S: AsRef<str>>(s: S, allocator: A) -> Self {
        let mut ret = Self {
            vec: Vector::new_in(NullTerminatorAllocator(allocator)),
        };
        ret.assign(s);

        ret
    }

    /// Creates a string with a custom allocator
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
            vec: Vector::new_in(NullTerminatorAllocator(allocator)),
        }
    }

    /// Assigns a string to a slice
    pub fn assign<S: AsRef<str>>(&mut self, buf: S) {
        self.reserve(buf.as_ref().len());

        // copy over and null terminate
        self.vec.append(buf.as_ref().as_bytes());

        // make sure the end is null-terminated
        unsafe { self.null_terminate() }
    }

    /// Returns the string as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Returns the string as a slice
    pub fn as_str(&self) -> &str {
        self
    }

    /// Returns the capacity of the string
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Returns true if the string is empty
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns true if the string is full to the capacity
    pub fn is_full(&self) -> bool {
        self.vec.is_full()
    }

    /// Returns the length of the string
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Pushes a new element into the string
    ///
    /// # Arguments
    ///
    /// `elem`: The new element
    pub fn push(&mut self, elem: char) {
        self.insert(self.len(), elem)
    }

    /// Pops an element off of the back of the array
    pub fn pop(&mut self) -> Option<char> {
        let elem = self.vec.pop().map(|elem| elem as char);

        // null terminate
        unsafe { self.null_terminate() }

        elem
    }

    /// Inserts a char into the string at an index.
    /// `index` must be less than or equal to `size`
    ///
    /// # Arguments
    ///
    /// `index`: The index to insert the char
    ///
    /// `elem`: The char to add to the string
    pub fn insert(&mut self, index: usize, elem: char) {
        self.vec.insert(index, elem as u8);

        // null terminate
        unsafe { self.null_terminate() }
    }

    /// Remove the char at the index and return it
    ///
    /// # Arguments
    ///
    /// `index`: The index of the element to remove
    pub fn remove(&mut self, index: usize) -> Option<char> {
        // account for null terminator
        if index >= self.len() {
            None
        } else {
            self.vec.remove(index).map(|elem| elem as char)
        }
    }

    /// Reserves space for chars within the string
    ///
    /// # Arguments
    ///
    /// `capacity`: The new capacity of the string
    pub fn reserve(&mut self, capacity: usize) {
        // make space for the null terminator
        self.vec.reserve(capacity + 1)
    }

    /// Null terminate the string.
    ///
    /// # Safety
    ///
    /// `end_ptr` must point to some valid memory (or null)
    unsafe fn null_terminate(&mut self) {
        // make sure the end is null-terminated
        if let Some(end) = self.vec.end_ptr.as_mut() {
            *end = 0;
        }
    }
}

impl<A: Allocator> AsRef<[u8]> for String<A> {
    fn as_ref(&self) -> &[u8] {
        self.vec.as_slice()
    }
}

impl<A: Allocator> AsRef<str> for String<A> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<A: Allocator + Clone> Clone for String<A> {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
        }
    }
}

impl<A: Allocator> Debug for String<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

impl<A: Allocator + Default> Default for String<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Allocator> Deref for String<A> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.vec) }
    }
}

impl<A: Allocator> DerefMut for String<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.vec) }
    }
}

impl<A: Allocator> Display for String<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<A: Allocator> PartialEq for String<A> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A: Allocator> Eq for String<A> {}

impl<A: Allocator + Default> From<&str> for String<A> {
    fn from(s: &str) -> Self {
        unsafe { Self::from_in(s, A::default()) }
    }
}

impl<A: Allocator + Default> FromStr for String<A> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl<A: Allocator> Hash<String<A>> for DefaultHash<String<A>> {
    fn hash(val: &String<A>) -> usize {
        DefaultHash::hash(val.as_str())
    }
}

unsafe impl<A: Allocator + Send> Send for String<A> {}
unsafe impl<A: Allocator + Sync> Sync for String<A> {}

#[cfg(test)]
mod test {
    use memoffset::offset_of;

    use crate::allocator::DefaultAllocator;
    use crate::string::DefaultString;

    use super::String;

    #[test]
    fn is_empty() {
        let mut s = DefaultString::new();
        assert!(s.is_empty());

        s.push('a');
        assert!(!s.is_empty());

        s.pop();
        assert!(s.is_empty());
    }

    #[test]
    fn is_full() {
        let mut s = DefaultString::new();
        assert!(s.is_full());

        s.push('a');
        assert!(s.is_full());

        s.pop();
        assert!(!s.is_full());
    }

    #[test]
    fn layout() {
        assert_eq!(offset_of!(String<DefaultAllocator>, vec), 0);
        assert_eq!(
            std::mem::size_of::<DefaultString>(),
            std::mem::size_of::<usize>() * 4
        )
    }

    #[test]
    fn from_str() {
        let s = DefaultString::from("abc");
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn push_chars() {
        let mut s = DefaultString::new();
        s.push('a');
        s.push('b');
        s.push('c');
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn pop_chars() {
        let mut s = DefaultString::from("ab");
        assert_eq!(s.pop(), Some('b'));
        assert_eq!(s.as_str(), "a");
    }

    #[test]
    fn push_pop() {
        let mut s = DefaultString::new();

        for _ in 0..5 {
            s.push('a');
            assert_eq!(s.pop(), Some('a'));
            assert_eq!(s.pop(), None);
        }

        assert!(s.is_empty());
    }

    #[test]
    fn insert() {
        let mut s = DefaultString::from("ab");
        s.insert(1, 'c');
        assert_eq!(s.as_str(), "acb");
    }

    #[test]
    #[should_panic]
    fn insert_out_of_bounds() {
        let mut s = DefaultString::from("ab");
        s.insert(3, 'c');
    }

    #[test]
    fn remove() {
        let mut s = DefaultString::from("ab");
        assert_eq!(s.remove(1), Some('b'));
        assert_eq!(s.remove(1), None);
        assert_eq!(s.as_str(), "a");
    }

    #[test]
    fn null_terminated() {
        let mut s = DefaultString::from("a");
        assert_eq!(unsafe { *s.vec.end_ptr }, 0);

        s.push('a');
        assert_eq!(unsafe { *s.vec.end_ptr }, 0);

        s.pop();
        assert_eq!(unsafe { *s.vec.end_ptr }, 0);

        s.assign("ab");
        assert_eq!(unsafe { *s.vec.end_ptr }, 0);

        s.insert(1, 'c');
        assert_eq!(unsafe { *s.vec.end_ptr }, 0);
    }

    #[test]
    fn equals() {
        let s1 = DefaultString::from("abcd");
        let s2 = DefaultString::from("abcd");
        let s3 = DefaultString::from("abce");

        assert!(s1.eq(&s2));
        assert!(s1.ne(&s3));
    }
}
