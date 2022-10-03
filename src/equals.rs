use std::marker::PhantomData;

/// A trait which takes two instances of something and returns true if they are equal
pub trait Equals<T> {
    /// Returns true if both instances are equal
    ///
    /// # Arguments
    ///
    /// `lhs`: The first instance
    ///
    /// `rhs`: The second instance
    fn equals(lhs: &T, rhs: &T) -> bool;
}

/// A struct which takes two instances of something and returns true if they are equal
pub struct EqualTo<T> {
    _marker: PhantomData<T>,
}

impl<T: Eq> Equals<T> for EqualTo<T> {
    fn equals(lhs: &T, rhs: &T) -> bool {
        lhs == rhs
    }
}
