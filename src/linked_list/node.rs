use std::ptr::null_mut;

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
    /// Create new node pointer without the previous and next nodes set
    #[inline]
    pub fn new(k: K, v: V) -> *mut Self {
        Box::into_raw(Box::new(Self {
            key: k,
            val: v,
            next: null_mut(),
            previous: null_mut(),
        }))
    }

    /// Return pointer to the previous node. Can be null.
    pub fn previous(&self) -> *mut Self {
        self.previous
    }

    /// Set the previous node pointer and set the next node pointer of the previous node, if any.
    ///
    /// This does set the next pointer on any existing previous node.
    #[inline]
    pub fn set_previous(&mut self, previous: *mut Self) {
        self.previous = previous;
        unsafe {
            if let Some(previous) = self.previous.as_mut() {
                previous.next = self;
            }
        }
    }

    /// Return pointer to the next node. Can be null.
    pub fn next(&self) -> *mut Self {
        self.next
    }

    /// Set the next node pointer and set the previous node pointer of the next node, if any.
    ///
    /// This does set the previous pointer on any existing next node.
    #[inline]
    pub fn set_next(&mut self, next: *mut Self) {
        self.next = next;
        unsafe {
            if let Some(next) = self.next.as_mut() {
                next.previous = self;
            }
        }
    }

    /// Remove node from the list, patching the previous and next values on the neighboring nodes
    #[inline]
    pub fn unlink(&mut self) {
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
