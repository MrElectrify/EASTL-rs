//!
//! Copyright (C) Warsaw Revamped. Any unauthorized use, modification, or distribution of any portion of this file is prohibited. All rights reserved.
//!

#[cfg(feature = "nightly")]
pub mod conformant;

use crate::allocator::{Allocator, DefaultAllocator};
use crate::fixed_pool::{with_overflow::FixedPoolWithOverflow, PoolAllocator};
use crate::list::node::{ListNode, ListNodeBase};
use crate::list::List;
use moveit::{new, New};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::{fmt, mem, slice};

/// A fixed list which uses the default allocator as an overflow.
pub type DefaultFixedList<T, const NODE_COUNT: usize> = FixedList<T, NODE_COUNT, DefaultAllocator>;

/// A list which allocates its nodes in-place. Note that there is not an implemented version of the
/// fixed list that does not support overflow. Note that this is not conformant because generics are
/// useless in rust :) use `conformant::FixedList` if you want 100% conformance
#[repr(C)]
#[allow(private_bounds)]
pub struct FixedList<T, const NODE_COUNT: usize, OverflowAllocator: Allocator> {
    base_list: List<T, FixedPoolWithOverflow<ListNode<T>, OverflowAllocator>>,
    // this should `technically` be conformant - `buffer` should be aligned to the alignment of
    // `ListNode<T>`...
    buffer: [MaybeUninit<ListNode<T>>; NODE_COUNT],
    // ... and then we add an extra node for the padding induced as shown in the conformant version
    _pad: MaybeUninit<ListNode<T>>,
}

#[allow(private_bounds)]
impl<T, const NODE_COUNT: usize, OverflowAllocator: Allocator>
    FixedList<T, NODE_COUNT, OverflowAllocator>
{
    /// Create a new, empty list.
    ///
    /// # Arguments
    /// `allocator`: The allocator to use
    ///
    /// # Safety
    /// The resulting list must not be moved.
    pub unsafe fn new_in(allocator: OverflowAllocator) -> impl New<Output = Self> {
        new::of(Self {
            base_list: List {
                node: ListNodeBase::default(),
                size: 0,
                allocator: FixedPoolWithOverflow::with_allocator(allocator),
                _holds_data: PhantomData,
            },
            // we actually don't care what the buffer contains
            buffer: MaybeUninit::uninit().assume_init(),
            _pad: MaybeUninit::uninit().assume_init(),
        })
        .with(|this| {
            let this = this.get_unchecked_mut();
            // TODO: better separation of concerns?
            this.base_list.init_sentinel_node();
            this.base_list.allocator.init(slice::from_raw_parts_mut(
                this.buffer.as_mut_ptr().cast(),
                this.buffer.len() * mem::size_of::<ListNode<T>>(),
            ));
        })
    }
}

#[allow(private_bounds)]
impl<T, const NODE_COUNT: usize, OverflowAllocator: Allocator + Default>
    FixedList<T, NODE_COUNT, OverflowAllocator>
{
    /// Create a new, empty list using the default overflow allocator.
    ///
    /// # Safety
    /// The resulting list must not be moved.
    pub unsafe fn new_with_default_overflow_allocator() -> impl New<Output = Self> {
        Self::new_in(OverflowAllocator::default())
    }
}

impl<T: fmt::Debug, const NODE_COUNT: usize, OverflowAllocator: Allocator> fmt::Debug
    for FixedList<T, NODE_COUNT, OverflowAllocator>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.base_list.fmt(f)
    }
}

impl<T, const NODE_COUNT: usize, OverflowAllocator: Allocator + Default> Deref
    for FixedList<T, NODE_COUNT, OverflowAllocator>
{
    type Target = List<T, FixedPoolWithOverflow<ListNode<T>, OverflowAllocator>>;

    fn deref(&self) -> &Self::Target {
        &self.base_list
    }
}

impl<T, const NODE_COUNT: usize, OverflowAllocator: Allocator + Default> DerefMut
    for FixedList<T, NODE_COUNT, OverflowAllocator>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_list
    }
}

#[cfg(test)]
mod test {
    use crate::allocator::DefaultAllocator;
    use crate::fixed_list::DefaultFixedList;
    use crate::fixed_pool::with_overflow::FixedPoolWithOverflow;
    use crate::list::node::{ListNode, ListNodeBase};
    use memoffset::offset_of;
    use moveit::moveit;
    use std::mem;

    #[repr(C, align(0x8))]
    struct Test {
        a: u32,
        b: u32,
        c: u32,
    }

    #[test]
    fn layout() {
        assert_eq!(offset_of!(DefaultFixedList<Test, 1>, base_list), 0x0);
        assert_eq!(
            offset_of!(DefaultFixedList<Test, 1>, buffer),
            mem::size_of::<ListNodeBase>()
                + mem::size_of::<usize>()
                + mem::size_of::<FixedPoolWithOverflow<ListNode<Test>, DefaultAllocator>>()
        );

        assert_eq!(
            mem::size_of::<DefaultFixedList<Test, 1>>(),
            mem::size_of::<ListNodeBase>()
                + mem::size_of::<usize>()
                + mem::size_of::<FixedPoolWithOverflow<ListNode<Test>, DefaultAllocator>>()
                + mem::size_of::<ListNode<Test>>() * 2
        );
    }

    #[test]
    fn fixed_alloc() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        list.push_front(12u32);

        // this allocation should be within the pool
        let pool_alloc = list.front().unwrap() as *const u32;
        assert!(
            pool_alloc >= list.base_list.allocator.pool_begin.cast()
                && pool_alloc <= list.base_list.allocator.pool_allocator.capacity.cast()
        );

        // push another
        list.push_front(13u32);
        // this allocation should not be within the pool
        let overflow_alloc = list.front().unwrap() as *const u32;
        assert!(
            overflow_alloc < list.base_list.allocator.pool_begin.cast()
                || overflow_alloc > list.base_list.allocator.pool_allocator.capacity.cast()
        );
    }

    // just copy the regular list tests
    #[test]
    fn empty() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<u32, 1>::new_in(DefaultAllocator::default()) };
        }
        assert!(list.is_empty());
    }

    #[test]
    fn size_empty() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<u32, 1>::new_with_default_overflow_allocator() };
        }
        assert_eq!(list.len(), 0);
        assert_eq!(list.size(), 0);
    }

    #[test]
    fn front_empty() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<u32, 1>::new_with_default_overflow_allocator() };
        }
        assert_eq!(list.front(), None)
    }

    #[test]
    fn back_empty() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<u32, 1>::new_with_default_overflow_allocator() };
        }
        assert_eq!(list.back(), None)
    }

    #[test]
    fn push_front() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        list.push_front(12u32);
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&12u32));
        assert_eq!(list.back(), Some(&12u32));
        list.push_front(6u32);
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&6u32));
        assert_eq!(list.back(), Some(&12u32));
    }

    #[test]
    fn push_front_owned_value() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_front(world.clone());
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&world));
        assert_eq!(list.back(), Some(&world));
        list.push_front(hello.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn push_back() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        assert_eq!(list.size(), 1);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&hello));
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.front(), Some(&hello));
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn modify_front() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        list.push_front("hello".to_string());
        let world = "world".to_string();
        *list.front_mut().unwrap() = world.clone();
        assert_eq!(list.back(), Some(&world));
    }

    #[test]
    fn clear() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        list.push_back(12u32);
        list.push_back(6u32);
        assert_eq!(list.size(), 2);
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
    }

    struct DropTest<'a> {
        r: &'a mut u32,
    }

    impl<'a> Drop for DropTest<'a> {
        fn drop(&mut self) {
            *self.r *= 2;
        }
    }

    #[test]
    fn drop() {
        let mut foo = 1;
        let mut bar = 1;
        {
            moveit! {
                let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
            }
            list.push_back(DropTest { r: &mut foo });
            list.push_back(DropTest { r: &mut bar });
        }
        assert_eq!(foo, 2);
        assert_eq!(bar, 2);
    }

    #[test]
    fn iter() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        let mut iter = list.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        list.push_back(12u32);
        list.push_back(6u32);
        let iter = list.iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        let vec = iter.collect::<Vec<&u32>>();
        assert_eq!(vec, vec![&12u32, &6u32]);
    }
    #[test]
    fn iter_mut() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        list.push_back(12u32);
        list.push_back(6u32);
        let mut iter = list.iter_mut();
        let first_val = iter.next().unwrap();
        assert_eq!(first_val, &mut 12u32);
        *first_val = 14u32;
        let mut iter = list.iter_mut();
        let first_val = iter.next().unwrap();
        assert_eq!(first_val, &mut 14u32);
        let last_val = iter.last().unwrap();
        assert_eq!(last_val, &mut 6u32);
    }

    #[test]
    fn pop_front() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.pop_front(), Some(hello));
        assert_eq!(list.size(), 1);
        assert_eq!(list.pop_front(), Some(world));
        assert!(list.is_empty());
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn pop_back() {
        moveit! {
            let mut list = unsafe { DefaultFixedList::<_, 1>::new_with_default_overflow_allocator() };
        }
        let hello = "hello".to_string();
        let world = "world".to_string();
        list.push_back(hello.clone());
        list.push_back(world.clone());
        assert_eq!(list.size(), 2);
        assert_eq!(list.pop_back(), Some(world));
        assert_eq!(list.size(), 1);
        assert_eq!(list.pop_back(), Some(hello));
        assert!(list.is_empty());
        assert_eq!(list.pop_front(), None);
    }
}
