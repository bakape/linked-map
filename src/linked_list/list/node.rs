use std::ptr::{null_mut, NonNull};

/// Linked list node containing value of type T
pub struct Node<K, V> {
    /// Previous node in the list
    previous: *mut Self,

    /// Next node in the list
    next: *mut Self,

    /// Key used for hashing
    pub key: K,

    /// Stored value
    pub val: V,
}

impl<K, V> Node<K, V> {
    /// Create new node pointer with both sibling nodes set to null
    #[inline]
    pub fn new(key: K, val: V) -> NonNull<Self> {
        let ptr = Box::into_raw(Box::new(Self {
            key,
            val,
            next: null_mut(),
            previous: null_mut(),
        }));
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Insert a node before this one
    #[inline]
    pub(super) fn insert_before(&mut self, mut node: NonNull<Self>) {
        unsafe {
            node.as_mut().next = self;
            node.as_mut().previous = self.previous;
        }

        self.previous = node.as_ptr();
    }
    /// Insert a node after this one
    #[inline]
    pub(super) fn insert_after(&mut self, mut node: NonNull<Self>) {
        unsafe {
            node.as_mut().next = self.next;
            node.as_mut().previous = self;
        }

        self.next = node.as_ptr();
    }

    /// Return pointer to the previous node. Can be null.
    #[inline]
    pub fn previous(&self) -> *mut Self {
        self.previous
    }

    /// Return pointer to the next node. Can be null.
    #[inline]
    pub fn next(&self) -> *mut Self {
        self.next
    }

    /// Remove node from the list, patching the previous and next values on the neighboring nodes.
    ///
    /// The pointers on the removed node are left as is, as they would be set as needed later on.
    #[inline]
    pub(super) fn remove(&mut self) {
        unsafe {
            if let Some(next) = self.next.as_mut() {
                next.previous = self.previous;
            }
            if let Some(previous) = self.previous.as_mut() {
                previous.next = self.next;
            }
        }

        // Erase links of removed node, just to be safe
        self.next = null_mut();
        self.previous = null_mut();
    }
}
