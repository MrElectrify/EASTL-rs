use crate::allocator::{Allocator, DefaultAllocator};
use crate::fixed_vector::allocator::FixedVectorAllocator;
use crate::vector::Vector;
use moveit::new::New;
use moveit::{new, MoveNew, MoveRef};
use std::ffi::c_void;
use std::fmt::Debug;
use std::mem::{size_of, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::{mem, ptr};

mod allocator;

/// Fixed vector with the default allocator.
pub type DefaultFixedVector<T, const NODE_COUNT: usize> =
    FixedVector<T, NODE_COUNT, DefaultAllocator>;

#[repr(C)]
pub struct FixedVector<T: Sized, const NODE_COUNT: usize, A: Allocator> {
    base_vec: Vector<T, FixedVectorAllocator<A>>,
    buffer: [MaybeUninit<T>; NODE_COUNT],
}

impl<T: Sized, const NODE_COUNT: usize, A: Allocator> FixedVector<T, NODE_COUNT, A> {
    /// Create a new fixed_vector with the given overflow allocator
    ///
    /// # Arguments
    /// `overflow_allocator`: The allocator to use for allocating overflowed elements in the base vector
    ///
    /// # Safety
    /// Raw pointer math
    pub unsafe fn new_in(overflow_allocator: A) -> impl New<Output = Self> {
        new::of(Self {
            base_vec: Vector::new_in(FixedVectorAllocator::new_with(overflow_allocator)),
            buffer: std::array::from_fn(|_| MaybeUninit::uninit().assume_init()),
        })
        .with(|this| {
            let this = this.get_unchecked_mut();
            this.init_base_vec();
        })
    }

    fn init_base_vec(&mut self) {
        self.base_vec.begin_ptr = self.buffer[0].as_mut_ptr();
        self.base_vec.end_ptr = self.buffer[0].as_mut_ptr();
        self.base_vec.capacity_ptr =
            (self.buffer[0].as_mut_ptr() as usize + (NODE_COUNT * size_of::<T>())) as *mut T;
        self.base_vec.allocator.pool_begin = self.buffer[0].as_mut_ptr() as *mut c_void;
    }
}

impl<T: Sized, const NODE_COUNT: usize, A: Allocator + Default> FixedVector<T, NODE_COUNT, A> {
    /// Create a new fixed_vector
    ///
    /// # Safety
    /// See `FixedVector::new_in`
    pub unsafe fn new() -> impl New<Output = Self> {
        Self::new_in(A::default())
    }
}

unsafe impl<T: Sized, const NODE_COUNT: usize, A: Allocator> MoveNew
    for FixedVector<T, NODE_COUNT, A>
{
    unsafe fn move_new(mut src: Pin<MoveRef<Self>>, this: Pin<&mut MaybeUninit<Self>>) {
        let this = this.get_unchecked_mut().assume_init_mut();
        let src = src.as_mut().get_unchecked_mut();
        // Swap the allocator over
        mem::swap(&mut this.base_vec.allocator, &mut src.base_vec.allocator);
        if !src.has_overflowed() {
            // We haven't overflowed, so we need to move the buffer
            mem::swap(&mut this.buffer, &mut src.buffer);
            // ... and re-init the base vec pointers point to it
            this.init_base_vec();
            // we have to fix the end pointer since it will be set to begin_ptr by init_base_vec
            this.base_vec.end_ptr = (this.base_vec.begin_ptr as usize
                + (src.base_vec.end_ptr as usize - src.base_vec.begin_ptr as usize))
                as *mut T;
        } else {
            // We have overflowed - we are not going to use `buffer` anymore so we might as well
            // leave it uninit - so we only copy over the base vec pointers
            this.base_vec.begin_ptr = src.base_vec.begin_ptr;
            this.base_vec.end_ptr = src.base_vec.end_ptr;
            this.base_vec.capacity_ptr = src.base_vec.capacity_ptr;
        }
        // zero `src` `begin_ptr` so any allocated data will not be dropped (we pretend like we never allocated it)
        src.base_vec.begin_ptr = null_mut();
    }
}

impl<T: Sized, const NODE_COUNT: usize, A: Allocator> FixedVector<T, NODE_COUNT, A> {
    /// Returns the max fixed size, which is the user-supplied NodeCount parameter
    pub fn max_size(&self) -> usize {
        NODE_COUNT
    }

    /// Returns true if the allocations spilled over into the overflow allocator. Meaningful only if overflow is enabled.
    pub fn has_overflowed(&self) -> bool {
        !ptr::eq(self.base_vec.begin_ptr, self.buffer[0].as_ptr())
    }
}

impl<T: Sized, const NODE_COUNT: usize, A: Allocator> AsRef<[T]> for FixedVector<T, NODE_COUNT, A> {
    fn as_ref(&self) -> &[T] {
        self.base_vec.as_ref()
    }
}

impl<T: Sized + Debug, const NODE_COUNT: usize, A: Allocator> Debug
    for FixedVector<T, NODE_COUNT, A>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.base_vec))
    }
}

impl<T: Sized + Debug, const NODE_COUNT: usize, A: Allocator> Deref
    for FixedVector<T, NODE_COUNT, A>
{
    type Target = Vector<T, FixedVectorAllocator<A>>;

    fn deref(&self) -> &Self::Target {
        &self.base_vec
    }
}

impl<T: Sized + Debug, const NODE_COUNT: usize, A: Allocator> DerefMut
    for FixedVector<T, NODE_COUNT, A>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_vec
    }
}

#[cfg(test)]
mod test {
    use crate::fixed_vector::DefaultFixedVector;
    use moveit::{moveit, MoveNew};
    use std::mem::MaybeUninit;
    use std::pin::Pin;

    #[test]
    fn push() {
        moveit! {
            let mut v = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        assert_eq!(v.len(), 0);
        assert!(!v.has_overflowed());
        assert!(!v.is_full());
        assert!(v.is_empty());
        v.push(64);
        assert_eq!(v.len(), 1);
        assert_eq!(v.as_slice()[0], 64);
        assert!(!v.has_overflowed());
        assert!(!v.is_full());
        assert!(!v.is_empty());
    }

    #[test]
    fn overflow() {
        moveit! {
            let mut v = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        for i in 0..12 {
            v.push(i);
        }
        assert_eq!(v.len(), 12);
        assert!(v.has_overflowed());
        assert_eq!(v.as_slice()[11], 11);
    }

    #[test]
    fn iter() {
        moveit! {
            let mut v = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.iter().sum::<u32>(), 6);
    }

    #[test]
    fn move_stack() {
        moveit! {
            let mut v = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        v.push(1);
        v.push(2);
        let mut target = MaybeUninit::<DefaultFixedVector<u32, 10>>::uninit();
        unsafe { MoveNew::move_new(v, Pin::new_unchecked(&mut target)) };
        let target = unsafe { target.assume_init_ref() };
        assert!(!target.is_full());
        assert!(!target.is_empty());
        assert_eq!(target.len(), 2);
        assert!(!target.has_overflowed());
    }

    #[test]
    fn move_overflow() {
        moveit! {
            let mut v = unsafe { DefaultFixedVector::<u32, 10>::new() };
        };
        for i in 0..12 {
            v.push(i);
        }
        let mut target = MaybeUninit::<DefaultFixedVector<u32, 10>>::uninit();
        unsafe { MoveNew::move_new(v, Pin::new_unchecked(&mut target)) };
        let target = unsafe { target.assume_init_ref() };
        assert_eq!(target.len(), 12);
        assert!(target.has_overflowed());
        assert_eq!(target.as_slice()[11], 11);
    }
}
