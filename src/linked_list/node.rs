use std::ptr::{null, null_mut, NonNull};

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

    /// Return pointer to the previous node. Can be null.
    pub fn previous(&self) -> *mut Self {
        self.previous
    }

    /// Return pointer to the next node. Can be null.
    pub fn next(&self) -> *mut Self {
        self.next
    }

    /// Remove node from the list, patching the previous and next values on this and the neighboring nodes
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
        self.next = null_mut();
        self.previous = null_mut();
    }
}
