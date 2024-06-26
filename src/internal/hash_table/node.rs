/// A node in a hashtable
#[repr(C)]
pub struct Node<K: PartialEq, V> {
    pub key: K,
    pub val: V,
    pub next: *mut Self,
}

impl<K: PartialEq, V> Node<K, V> {
    /// Creates the node from a key/value
    ///
    /// # Arguments
    ///
    /// `key`: The key
    ///
    /// `val`: The value
    ///
    /// `next`: The next linked node in the list
    pub fn new(key: K, value: V, next: *mut Self) -> Self {
        Self {
            key,
            val: value,
            next,
        }
    }

    /// Returns the key of the node
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Returns the value of the node
    pub fn value(&self) -> &V {
        &self.val
    }

    /// Returns the value of the node
    pub fn value_mut(&mut self) -> &mut V {
        &mut self.val
    }

    /// Returns the next node following this node
    pub fn next(&self) -> Option<&Self> {
        unsafe { self.next.as_ref() }
    }

    /// Returns the next node following this node
    pub fn next_mut(&mut self) -> Option<&mut Self> {
        unsafe { self.next.as_mut() }
    }
}
