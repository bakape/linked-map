pub mod node;
mod tests;

mod cursor;
pub use cursor::{Cursor, CursorMut};

use node::Node;
use std::ptr::{null_mut, NonNull};

/// Doubly-linked list with cursor iteration support
pub struct LinkedList<K, V> {
    /// First node of list. null, if list is empty.
    pub head: *mut Node<K, V>,

    /// Last node of the list. null, if list is empty.
    pub tail: *mut Node<K, V>,
}

impl<K, V> Drop for LinkedList<K, V> {
    fn drop(&mut self) {
        let mut next = self.head;
        while !next.is_null() {
            let b = unsafe { Box::from_raw(next) };
            next = b.next();
        }
    }
}

impl<K, V> LinkedList<K, V> {
    /// Create new empty list
    #[inline]
    pub fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    /// Append a node to the end of the list and return a pointer to it
    #[inline]
    pub fn append(&mut self, k: K, v: V) -> NonNull<Node<K, V>> {
        let node = Node::insert(k, v, self.tail, null_mut());
        if self.head.is_null() {
            self.head = node.as_ptr();
        }
        node
    }

    /// Prepend a node to the start the list and return a pointer to it
    #[inline]
    pub fn prepend(&mut self, k: K, v: V) -> NonNull<Node<K, V>> {
        let node = Node::insert(k, v, null_mut(), self.head);
        if self.tail.is_null() {
            self.tail = node.as_ptr();
        }
        node
    }
}
