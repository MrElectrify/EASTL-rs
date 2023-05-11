use std::convert::Infallible;
use std::str::FromStr;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use crate::allocator::DefaultAllocator;
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
    vec: Vector<u8, A>,
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
    pub unsafe fn from_in(s: &str, allocator: A) -> Self {
        Self {
            vec: Vector::from_in(s.as_bytes(), allocator),
        }
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
            vec: Vector::new_in(allocator),
        }
    }

    /// Assigns a string to a slice
    pub fn assign<S: AsRef<str>>(&mut self, buf: S) {
        self.vec.assign(buf.as_ref().as_bytes())
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
        self.vec.push(elem as u8)
    }

    /// Pops an element off of the back of the array
    pub fn pop(&mut self) -> Option<char> {
        self.vec.pop().map(|elem| elem as char)
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
        self.vec.insert(index, elem as u8)
    }

    /// Remove the char at the index and return it
    ///
    /// # Arguments
    ///
    /// `index`: The index of the element to remove
    pub fn remove(&mut self, index: usize) -> Option<char> {
        self.vec.remove(index).map(|elem| elem as char)
    }

    /// Reserves space for chars within the string
    ///
    /// # Arguments
    ///
    /// `capacity`: The new capacity of the string
    pub fn reserve(&mut self, capacity: usize) {
        self.vec.reserve(capacity)
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

impl<A: Allocator + Default> From<&str> for String<A> {
    fn from(s: &str) -> Self {
        Self {
            vec: Vector::from(s.as_bytes()),
        }
    }
}

impl<A: Allocator + Default> FromStr for String<A> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            vec: Vector::from(s.as_bytes()),
        })
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
        assert_eq!(s.capacity(), 3);
    }

    #[test]
    fn push_chars() {
        let mut s = DefaultString::new();
        s.push('a');
        s.push('b');
        s.push('c');
        assert_eq!(s.as_str(), "abc");
        assert_eq!(s.capacity(), 4);
    }

    #[test]
    fn pop_chars() {
        let mut s = DefaultString::from("ab");
        assert_eq!(s.pop(), Some('b'));
        assert_eq!(s.as_str(), "a");
    }

    #[test]
    fn insert() {
        let mut s = DefaultString::from("ab");
        s.insert(1, 'c');
        assert_eq!(s.as_str(), "acb");
        assert_eq!(s.capacity(), 4);
    }

    #[test]
    fn remove() {
        let mut s = DefaultString::from("ab");
        assert_eq!(s.remove(1), Some('b'));
        assert_eq!(s.as_str(), "a");
    }
}
