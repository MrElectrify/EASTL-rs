use std::{fmt::Debug, marker::PhantomData};

/// The color of a red-black tree node
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Color {
    Red = 0,
    Black = 1,
}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Red,
            1 => Self::Black,
            _ => panic!("Bad color"),
        }
    }
}

/// A parent-color compressed pair. Combined, the
/// pair takes `std::mem::size_of::<usize>()` bytes
#[repr(C)]
pub(crate) struct ParentColor<K, V> {
    raw_ptr: usize,
    _ignore_key: PhantomData<K>,
    _ignore_value: PhantomData<V>,
}

impl<K, V> ParentColor<K, V> {
    /// Creates a new parent-color compressed pair
    pub fn _new(color: Color, parent_ptr: *mut Node<K, V>) -> Self {
        let mut this = Self::default();
        this._set_color(color);
        this._set_ptr(parent_ptr);
        this
    }

    /// Fetches the color of the parent-color compressed pair
    pub fn color(&self) -> Color {
        Color::from((self.raw_ptr & 1) as u32)
    }

    /// Fetches the pointer stored in the parent-color compressed pair
    pub fn ptr(&self) -> *mut Node<K, V> {
        (self.raw_ptr & !1) as *mut Node<K, V>
    }

    /// Sets the color of the node
    ///
    /// # Arguments
    ///
    /// `color`: The color of the node
    pub fn _set_color(&mut self, color: Color) {
        self.raw_ptr = (self.raw_ptr & !1) | color as usize;
    }

    /// Sets the parent pointer of the node
    ///
    /// # Arguments
    ///
    /// `parent_ptr`: The parent pointer of the node
    pub fn _set_ptr(&mut self, parent_ptr: *mut Node<K, V>) {
        self.raw_ptr = (self.raw_ptr & 1) | parent_ptr as usize;
    }
}

impl<K, V> Debug for ParentColor<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{:?}", self.ptr(), self.color())
    }
}

impl<K, V> Default for ParentColor<K, V> {
    fn default() -> Self {
        Self {
            raw_ptr: 0,
            _ignore_key: PhantomData,
            _ignore_value: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Node<K, V> {
    pub right: *mut Node<K, V>,
    pub left: *mut Node<K, V>,
    pub parent: ParentColor<K, V>,
    key: K,
    val: V,
}

impl<K: Default, V: Default> Default for Node<K, V> {
    fn default() -> Self {
        Self {
            right: std::ptr::null_mut(),
            left: std::ptr::null_mut(),
            parent: ParentColor::default(),
            key: K::default(),
            val: V::default(),
        }
    }
}

impl<K, V> Node<K, V> {
    // Very UB - Self will be moved out and left/right parent pointers will be wrong. Fix this!
    /* pub fn _new(
        left: *mut Self,
        right: *mut Self,
        parent: *mut Self,
        color: Color,
        key: K,
        val: V,
    ) -> Self {
        let parent = ParentColor::_new(color, parent);
        let mut this = Self {
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent,
            key,
            val,
        };
        this._set_left(left);
        this._set_right(right);
        this
    } */

    /// The color of the node
    pub fn _color(&self) -> Color {
        self.parent.color()
    }

    /// Sets the color of the node
    ///
    /// # Arguments
    ///
    /// `color`: The new color of the node
    pub fn _set_color(&mut self, color: Color) {
        self.parent._set_color(color)
    }

    /// The left child of the node
    pub fn left(&self) -> Option<&mut Self> {
        unsafe { self.left.as_mut() }
    }

    /// Sets the left child of the node, returning the
    /// old left child. Updates the child node as well
    ///
    /// # Arguments
    ///
    /// `left`: The new left child of the node
    pub fn _set_left(&mut self, left: *mut Self) -> *mut Self {
        let old_left = self.left;
        self.left = left;
        unsafe { self.left.as_mut() }.map(|left| left._set_parent(self));
        old_left
    }

    /// The parent of the node
    pub fn _parent(&self) -> Option<&mut Self> {
        unsafe { self.parent.ptr().as_mut() }
    }

    /// Sets the parent of the node, returning the
    /// old parent
    ///
    /// # Arguments
    ///
    /// `parent`: The new parent of the node
    pub fn _set_parent(&mut self, parent: *mut Self) -> *mut Self {
        let old_parent = self.parent.ptr();
        self.parent._set_ptr(parent);
        old_parent
    }

    /// The right child of the node
    pub fn right(&self) -> Option<&mut Self> {
        unsafe { self.right.as_mut() }
    }

    /// Sets the right child of the node, returning the
    /// old right child
    ///
    /// # Arguments
    ///
    /// `right`: The new right child of the node
    pub fn _set_right(&mut self, right: *mut Self) -> *mut Self {
        let old_right = self.right;
        self.right = right;
        unsafe { self.right.as_mut() }.map(|right| right._set_parent(self));
        old_right
    }

    /// The key stored in the node
    pub fn key(&self) -> &K {
        &self.key
    }

    /// The value stored in the node
    pub fn val(&self) -> &V {
        &self.val
    }

    /// The value stored in the node
    pub fn val_mut(&mut self) -> &mut V {
        &mut self.val
    }
}

#[cfg(test)]
mod test {
    use super::{Color, Node, ParentColor};

    #[test]
    fn empty_parent() {
        let parent_color = ParentColor::<u32, u32>::default();

        assert_eq!(parent_color.color(), Color::Red);
        assert!(parent_color.ptr().is_null());
    }

    #[test]
    fn non_empty_black_ptr() {
        let mut node = Node::<u32, u32>::default();
        let parent_color = ParentColor::_new(Color::Black, &mut node);

        assert_eq!(parent_color.color(), Color::Black);
        assert_eq!(parent_color.ptr(), &mut node as *mut Node<u32, u32>);
    }

    #[test]
    fn non_empty_red_ptr() {
        let mut node = Node::<u32, u32>::default();
        let parent_color = ParentColor::_new(Color::Red, &mut node);

        assert_eq!(parent_color.color(), Color::Red);
        assert_eq!(parent_color.ptr(), &mut node as *mut Node<u32, u32>);
    }

    #[test]
    fn empty_node() {
        let node = Node::<u32, u32>::default();

        assert!(node.left().is_none());
        assert!(node.right().is_none());
        assert!(node._parent().is_none());
        assert_eq!(node._color(), Color::Red);
        assert_eq!(node.val(), &0);
    }

    /* #[test]
    fn non_empty_node() {
        let mut left = Node::<u32, u32>::default();
        let mut right = Node::<u32, u32>::default();
        let parent = Node::<u32, u32>::_new(
            &mut left,
            &mut right,
            std::ptr::null_mut(),
            Color::Red,
            5,
            6,
        );

        assert_eq!(parent._color(), Color::Red);
        assert_eq!(
            parent.left().unwrap()._parent().unwrap() as *const Node::<u32, u32>,
            &parent
        );
        assert_eq!(
            parent.right().unwrap()._parent().unwrap() as *const Node::<u32, u32>,
            &parent
        );
        assert_eq!(parent.key(), &5);
        assert_eq!(parent.val(), &6);
    } */
}
