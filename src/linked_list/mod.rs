pub mod node;
mod tests;

mod cursor;
pub use cursor::{Cursor, CursorMut};

use node::Node;
use std::ptr::null_mut;

/// Doubly-linked list with cursor iteration support
pub(crate) struct LinkedList<K, V> {
    /// First node of list. null, if list is empty.
    head: *mut Node<K, V>,

    /// Last node of the list. null, if list is empty.
    tail: *mut Node<K, V>,
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
    pub(crate) fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }
}
