use crate::{
    allocator::{Allocator, DefaultAllocator},
    compare::{Compare, Less},
};

use self::node::Node;

mod node;

#[repr(C)]
pub struct RBTree<K: Eq, V, C: Compare<K> = Less<K>, A: Allocator = DefaultAllocator> {
    /// A 1-size functor in C++
    compare: C,
    /// Real EASTL uses a node without a K/V pair
    /// here, but that would mean we would need some
    /// base node as well. Splitting them up also
    /// just makes sense
    begin: *mut Node<K, V>,
    end: *mut Node<K, V>,
    parent: *mut Node<K, V>,
    size: usize,
    allocator: A,
}

impl<K: Eq, V, C: Compare<K> + Default, A: Allocator + Default> RBTree<K, V, C, A> {
    /// Returns a default new red-black tree
    fn new() -> Self {
        Self {
            compare: C::default(),
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            size: 0,
            allocator: A::default(),
        }
    }
}

impl<K: Eq, V, C: Compare<K> + Default, A: Allocator + Default> Default for RBTree<K, V, C, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq, V, C: Compare<K>, A: Allocator> Drop for RBTree<K, V, C, A> {
    fn drop(&mut self) {
        self.free_nodes()
    }
}

impl<K: Eq, V, C: Compare<K> + Default, A: Allocator> RBTree<K, V, C, A> {
    /// Constructs a red-black tree using a specified allocator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            compare: C::default(),
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            size: 0,
            allocator,
        }
    }
}

impl<K: Eq, V, C: Compare<K>, A: Allocator + Default> RBTree<K, V, C, A> {
    /// Constructs a red-black tree using a specified comparator
    ///
    /// # Arguments
    ///
    /// `compare`: The comparator
    pub fn with_compare(compare: C) -> Self {
        Self {
            compare,
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            size: 0,
            allocator: A::default(),
        }
    }
}

impl<K: Eq, V, C: Compare<K>, A: Allocator> RBTree<K, V, C, A> {
    /// Constructs a red-black tree using a specified allocator
    /// and comparator
    ///
    /// # Arguments
    ///
    /// `allocator`: The allocator
    ///
    /// `compare`: The comparator
    pub fn with_allocator_and_compare(allocator: A, compare: C) -> Self {
        Self {
            compare,
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            size: 0,
            allocator,
        }
    }

    /// Clears the red-black tree, removing all key-value pairs
    pub fn clear(&mut self) {
        self.free_nodes()
    }

    /// Returns true if the red-black tree contains a pair indexed
    /// by the given key
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Fetches the value indexed by the key in the tree
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get(&self, key: &K) -> Option<&V> {
        self.find_in_tree(key).map(|n| n.val())
    }

    /// Fetches the value indexed by the key in the tree
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.find_in_tree(key).map(|n| n.val_mut())
    }

    /// Inserts a key-value pair into the red-black tree
    ///
    /// # Arguments
    ///
    /// `key`: The key to insert and index by
    ///
    /// `value`: The value to insert
    pub fn _insert(&mut self, key: K, _value: V) -> Option<V> {
        let _insertion_position = self._find_insertion_position(&key);
        unimplemented!()
    }

    /// Returns true if the red-black tree contains no elements
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements in the red-black tree
    pub fn len(&self) -> usize {
        self.size
    }

    /// Removes a key-value pair from the red-black tree,
    /// returning the element if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn _remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, val)| val)
    }

    /// Removes a key-value pair from the red-black tree,
    /// returning the pair if it was found
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    pub fn remove_entry(&mut self, _key: &K) -> Option<(K, V)> {
        unimplemented!()
    }

    /// Finds the node in the tree given the head and key
    ///
    /// # Arguments
    ///
    /// `head`: The head node of the tree
    ///
    /// `key`: The key to index the pair
    fn find_in_tree(&self, key: &K) -> Option<&mut Node<K, V>> {
        let mut current_node = self.parent();
        while let Some(node) = current_node {
            if C::compare(key, node.key()) {
                current_node = node.left();
            // if the key !< node and node !< key they must be equal
            } else if !C::compare(node.key(), key) {
                return Some(node);
            } else {
                current_node = node.right();
            }
        }
        None
    }

    /// Finds the position to insert a new key-value pair
    ///
    /// # Arguments
    ///
    /// `key`: The key to index the pair
    fn _find_insertion_position(&self, key: &K) -> Option<&mut Node<K, V>> {
        let mut current_node = self.parent;
        let mut prev_node = std::ptr::null_mut();
        let mut _is_key_less_than_node = false;
        while let Some(node) = unsafe { current_node.as_mut() } {
            prev_node = current_node;
            _is_key_less_than_node = C::compare(key, node.key());
            if _is_key_less_than_node {
                current_node = node.left;
            } else {
                current_node = node.right;
            }
        }

        unsafe { prev_node.as_mut() }
    }

    /// Frees a node and its children
    ///
    /// # Arguments
    ///
    /// `root`: The root node of the tree
    fn free_tree(&mut self, root: &mut Node<K, V>) {
        if let Some(left) = root.left() {
            self.free_tree(left)
        }
        if let Some(right) = root.right() {
            self.free_tree(right)
        }
        // deallocate the current node
        unsafe { self.allocator.deallocate(root, 1) }
    }

    /// Frees all of the nodes in the tree
    fn free_nodes(&mut self) {
        if let Some(node) = unsafe { self.parent.as_mut() } {
            self.free_tree(node)
        }
    }

    /// Returns the parent node
    fn parent(&self) -> Option<&mut Node<K, V>> {
        unsafe { self.parent.as_mut() }
    }

    /// Returns the beginning (lowest) node
    fn _begin(&self) -> Option<&mut Node<K, V>> {
        unsafe { self.begin.as_mut() }
    }

    /// Returns the end (highest) node
    fn _end(&self) -> Option<&mut Node<K, V>> {
        unsafe { self.end.as_mut() }
    }
}

#[cfg(test)]
mod test {
    use memoffset::offset_of;

    use super::RBTree;

    #[test]
    fn layout() {
        assert_eq!(
            offset_of!(RBTree<u32, u32>, begin),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            offset_of!(RBTree<u32, u32>, end),
            std::mem::size_of::<usize>() * 2
        );
        assert_eq!(
            offset_of!(RBTree<u32, u32>, parent),
            std::mem::size_of::<usize>() * 3
        );
        assert_eq!(
            offset_of!(RBTree<u32, u32>, size),
            std::mem::size_of::<usize>() * 4
        );
        assert_eq!(
            offset_of!(RBTree<u32, u32>, allocator),
            std::mem::size_of::<usize>() * 5
        );
        assert_eq!(
            std::mem::size_of::<RBTree<u32, u32>>(),
            std::mem::size_of::<usize>() * 6
        );
    }

    #[test]
    fn default() {
        let rb_tree = RBTree::<u32, u32>::default();
        assert_eq!(rb_tree.begin, std::ptr::null_mut());
        assert_eq!(rb_tree.end, std::ptr::null_mut());
        assert_eq!(rb_tree.parent, std::ptr::null_mut());
        assert_eq!(rb_tree.size, 0);

        assert_eq!(rb_tree.len(), 0);
        assert!(rb_tree.is_empty());
    }
}
