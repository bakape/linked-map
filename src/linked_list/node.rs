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
    /// Create new node pointer and insert it into an existing list.
    /// The previous and next nodes can be null.
    #[inline]
    pub fn insert(key: K, val: V, previous: *mut Self, next: *mut Self) -> NonNull<Self> {
        let ptr = Box::into_raw(Box::new(Self {
            key,
            val,
            next,
            previous,
        }));
        unsafe {
            if let Some(next) = next.as_mut() {
                next.previous = ptr;
            }
            if let Some(previous) = previous.as_mut() {
                previous.next = ptr;
            }

            NonNull::new_unchecked(ptr)
        }
    }

    /// Insert a node before this one
    pub fn insert_before(&mut self, mut node: NonNull<Self>) {
        unsafe {
            node.as_mut().next = self;
            node.as_mut().previous = self.previous;
        }

        self.previous = node.as_ptr();
    }

    /// Insert a node after this one
    pub fn insert_after(&mut self, mut node: NonNull<Self>) {
        unsafe {
            node.as_mut().next = self.next;
            node.as_mut().previous = self;
        }

        self.next = node.as_ptr();
    }

    /// Return pointer to the previous node. Can be null.
    pub fn previous(&self) -> *mut Self {
        self.previous
    }

    /// Return pointer to the next node. Can be null.
    pub fn next(&self) -> *mut Self {
        self.next
    }

    /// Remove node from the list, patching the previous and next values on the neighboring nodes.
    ///
    /// The pointers on the removed node are left as is, as they would be set as needed later on.
    #[inline]
    pub fn remove(&mut self) {
        unsafe {
            if let Some(next) = self.next.as_mut() {
                next.previous = self.previous;
            }
            if let Some(previous) = self.previous.as_mut() {
                previous.next = self.next;
            }
        }
    }
}
