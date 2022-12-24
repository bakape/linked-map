pub mod node;
pub use node::Node;

use std::ptr::{null_mut, NonNull};

/// Doubly-linked list with cursor iteration support
pub struct LinkedList<K, V> {
    /// First node of list. `null`, if list is empty.
    head: *mut Node<K, V>,

    /// Last node of the list. `null`, if list is empty.
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

impl<K, V> Default for LinkedList<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> LinkedList<K, V> {
    /// Create new empty list
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    /// Return pointer to list head
    pub fn head(&self) -> *mut Node<K, V> {
        self.head
    }

    /// Return pointer to list tail
    pub fn tail(&self) -> *mut Node<K, V> {
        self.tail
    }

    /// Append a node to the end of the list and return a pointer to it
    #[inline]
    pub fn append(&mut self, k: K, v: V) -> NonNull<Node<K, V>> {
        let node = Node::new(k, v);
        match unsafe { self.tail.as_mut() } {
            Some(tail) => unsafe {
                self.insert_after(node, tail.into());
            },
            None => {
                self.head = node.as_ptr();
                self.tail == node.as_ptr();
            }
        }
        node
    }

    /// Prepend a node to the start the list and return a pointer to it
    #[inline]
    pub fn prepend(&mut self, k: K, v: V) -> NonNull<Node<K, V>> {
        let node = Node::new(k, v);
        match unsafe { self.head.as_mut() } {
            Some(head) => unsafe {
                self.insert_before(node, head.into());
            },
            None => {
                self.head = node.as_ptr();
                self.tail == node.as_ptr();
            }
        }
        node
    }

    /// Move the node to the front of the list
    pub fn move_to_front(&mut self, node: NonNull<Node<K, V>>) {
        let node = unsafe { node.as_mut() };
        if self.head == node {
            return;
        }

        if self.tail == node {
            self.tail = node.previous();
        }
        node.remove();

        unsafe { self.head.as_mut() }
            .expect("list head to be set")
            .insert_before(node.into());
        self.head = node;
    }

    /// Move the node to the back of the list
    pub fn move_to_back(&mut self, node: NonNull<Node<K, V>>) {
        let node = unsafe { node.as_mut() };
        if self.tail == node {
            return;
        }

        if self.head == node {
            self.head = node.next();
        }
        node.remove();

        unsafe { self.tail.as_mut() }
            .expect("list tail to be set")
            .insert_before(node.into());
        self.tail = node;
    }

    /// Remove a node from the list
    pub fn remove(&mut self, node: NonNull<Node<K, V>>) {
        let node = unsafe { node.as_mut() };
        if self.head == node {
            self.head = node.next();
        }
        if self.tail == node {
            self.tail = node.previous();
        }
        node.remove();
    }

    /// Insert a node before a different node.
    ///
    /// # SAFETY
    ///
    /// `node` and `before` must not be the same node.
    #[inline]
    pub unsafe fn insert_before(&mut self, node: NonNull<Node<K, V>>, before: NonNull<Node<K, V>>) {
        let was_head = self.head == before.as_ptr();
        before.as_mut().insert_before(node);
        if was_head {
            self.head = node.as_ptr();
        }
    }

    /// Insert a node after a different node.
    ///
    /// # SAFETY
    ///
    /// `node` and `after` must not be the same node.
    #[inline]
    pub unsafe fn insert_after(&mut self, node: NonNull<Node<K, V>>, after: NonNull<Node<K, V>>) {
        let was_tail = self.tail == after.as_ptr();
        after.as_mut().insert_after(node);
        if was_tail {
            self.tail = node.as_ptr();
        }
    }
}
