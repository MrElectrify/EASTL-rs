use std::mem;

/// Rotates the pair of iterators towards `next`.
pub unsafe fn rotate<'a, I: 'a, I1: Iterator<Item = &'a mut I>, I2: Iterator<Item = &'a mut I>>(
    mut current: I1,
    mut next: I2,
) {
    while let (Some(current), Some(next)) = (current.next(), next.next()) {
        mem::swap(current, next)
    }
}
