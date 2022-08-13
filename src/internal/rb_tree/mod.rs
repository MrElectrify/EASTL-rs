mod node;

#[repr(C)]
pub struct RBTree<K, V> {
    head: node::Node<K, V>,
}
