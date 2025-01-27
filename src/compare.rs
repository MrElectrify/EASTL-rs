use std::marker::PhantomData;

/// A comparator trait which compares two nodes
pub trait Compare<T> {
    /// Compare two values, and return true if
    /// `left` is lesser to `right`
    fn compare(left: &T, right: &T) -> bool;
}

/// A struct that implements `Compare` for `T`, and
/// returns true if `left` > `right`
pub struct Greater<T> {
    _pad: u8,
    _marker: PhantomData<T>,
}

impl<T: PartialOrd> Compare<T> for Greater<T> {
    fn compare(left: &T, right: &T) -> bool {
        left > right
    }
}

impl<T> Default for Greater<T> {
    fn default() -> Self {
        Self {
            _pad: 0,
            _marker: PhantomData,
        }
    }
}

/// A struct that implements `Compare` for `T`, and
/// returns true if `left` < `right`
pub struct Less<T> {
    _pad: u8,
    _marker: PhantomData<T>,
}

impl<T: PartialOrd> Less<T> {
    pub const fn new() -> Self {
        Self {
            _pad: 0,
            _marker: PhantomData,
        }
    }
}

impl<T: PartialOrd> Compare<T> for Less<T> {
    fn compare(left: &T, right: &T) -> bool {
        left < right
    }
}

impl<T: PartialOrd> Default for Less<T> {
    fn default() -> Self {
        Self::new()
    }
}
