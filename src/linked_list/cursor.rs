use hashbrown::hash_map::{Entry, OccupiedEntry};

use crate::LinkedMap;

use super::node::Node;
use paste::paste;
use std::{
    hash::{BuildHasher, Hash},
    ptr::{null, null_mut, NonNull},
};

/// Implement functionality common to both mutable and immutable cursors
macro_rules! impl_common {
    ($key_value:ty, $iterator:ident) => {
        /// Helper for getting the current node
        #[inline]
        fn current(&self) -> Option<&mut Node<K, V>> {
            unsafe { self.current.as_mut() }
        }

        /// Navigate to the start of the linked list
        pub fn to_start(&mut self) {
            self.current = self.parent.list.head;
        }

        /// Navigate to the end of the linked list
        pub fn to_end(&mut self) {
            self.current = self.parent.list.tail;
        }

        /// Returns a reference to the current node's key in the map.
        /// Only returns None, if the current linked list is empty.
        pub fn key(&self) -> Option<&K> {
            self.current().map(|n| &n.key)
        }

        /// Return a reference to the current node's key-value pair.
        /// Only returns None, if the list is empty.
        pub fn key_value(&self) -> Option<$key_value> {
            Self::map_ptr(self.current)
        }

        /// Return the number of elements in the parent [LinkedMap]
        pub fn len(&self) -> usize {
            self.parent.map.len()
        }

        /// Return, if the parent [LinkedMap] is empty
        pub fn is_empty(&self) -> bool {
            self.parent.map.is_empty()
        }

        /// Returns the number of elements the parent [LinkedMap] can hold without reallocating
        pub fn capacity(&self) -> usize {
            self.parent.map.capacity()
        }

        /// Try to advances cursor to the next node and return the key and value of that node
        #[inline]
        #[allow(clippy::should_implement_trait)]
        pub fn next(&mut self) -> Option<$key_value> {
            unsafe { self.current.as_ref() }.and_then(|current| {
                unsafe { current.next().as_mut() }.map(|next| {
                    self.current = next;
                    Self::map_non_null(next.into())
                })
            })
        }

        /// Try to move cursor to the next previous and return the key and value of that node
        #[inline]
        pub fn previous(&mut self) -> Option<$key_value> {
            unsafe { self.current.as_ref() }.and_then(|current| {
                unsafe { current.previous().as_mut() }.map(|prev| {
                    self.current = prev;
                    Self::map_non_null(prev.into())
                })
            })
        }

        /// Try to navigate to the given key.
        /// Returns [None], if no such key found.
        pub fn to_key(&mut self, key: &K) -> Option<$key_value> {
            self.parent.map.get(key).map(|n| {
                let n = *n;
                self.current = n;
                Self::map_non_null(unsafe { NonNull::new_unchecked(n) })
            })
        }

        /// Get the key and value of the next node (if any), without advancing the cursor
        pub fn peek_next(&self) -> Option<$key_value> {
            self.current().and_then(|n| Self::map_ptr(n.next()))
        }

        /// Get the key and value of the previous node (if any), without advancing the cursor
        pub fn peek_previous(&self) -> Option<$key_value> {
            self.current().and_then(|n| Self::map_ptr(n.previous()))
        }

        /// Iterate the list towards the tail
        pub fn iter(self) -> impl Iterator<Item = $key_value> {
            paste! {
                crate::iter::[< $iterator Forward>]::new(self)
            }
        }

        /// Iterate the list towards the head
        pub fn iter_rev(self) -> impl Iterator<Item = $key_value> {
            paste! {
                crate::iter::[< $iterator Backward>]::new(self)
            }
        }

        /// Navigate cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
        ///
        /// If no node is currently saved, returns `false`.
        pub fn resume(&mut self) -> bool {
            if self.parent.saved.is_null() {
                false
            } else {
                self.current = self.parent.saved;
                true
            }
        }
    };
}

/// Enables safe linked list traversal
pub struct Cursor<'a, K, V, S> {
    /// Parent [LinkedMap]
    parent: &'a LinkedMap<K, V, S>,

    /// Node the cursor is currently at. Can be null, if parent is empty.
    current: *mut Node<K, V>,
}

impl<'a, K, V, S> Cursor<'a, K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    impl_common! {(&'a K, &'a V), Iter}

    /// Create a cursor over the passed list, setting the cursor position to the passed node.
    ///
    /// `position` must belong to the parent and be null, only if the parent is empty.
    #[inline]
    pub(crate) unsafe fn new(parent: &'a LinkedMap<K, V, S>, position: *mut Node<K, V>) -> Self {
        Self {
            parent,
            current: position,
        }
    }

    /// Return a reference to the current node's value.
    /// Only returns None, if the list is empty.
    pub fn value(&self) -> Option<&V> {
        self.current().map(|n| &n.val)
    }

    /// Map pointer to key-value reference pair
    #[inline]
    pub(crate) fn map_ptr(node: *mut Node<K, V>) -> Option<(&'a K, &'a V)> {
        unsafe { node.as_ref() }.map(|n| (&n.key, &n.val))
    }

    /// Map NonNull pointer to key-value reference pair
    #[inline]
    fn map_non_null(node: NonNull<Node<K, V>>) -> (&'a K, &'a V) {
        let n = unsafe { node.as_ref() };
        (&n.key, &n.val)
    }
}

impl<'a, K, V, S> Clone for Cursor<'a, K, V, S> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent,
            current: self.current,
        }
    }
}

/// Enables safe linked list traversal and mutation
pub struct CursorMut<'a, K, V, S> {
    /// Parent [LinkedMap]
    parent: &'a mut LinkedMap<K, V, S>,

    /// Node the cursor is currently at. Can be null, if parent is empty.
    current: *mut Node<K, V>,
}

impl<'a, K, V, S> CursorMut<'a, K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    impl_common! {(&'a K, &'a mut V), IterMut}

    /// Create a cursor over the passed list, setting the cursor position to the passed node.
    ///
    /// `position` must belong to the parent and be null, only if the parent is empty.
    #[inline]
    pub(crate) unsafe fn new(
        parent: &'a mut LinkedMap<K, V, S>,
        position: *mut Node<K, V>,
    ) -> Self {
        Self {
            parent,
            current: position,
        }
    }

    /// Return a reference to the current node's value.
    /// Only returns None, if the list is empty.
    pub fn value(&mut self) -> Option<&mut V> {
        unsafe { self.current.as_mut() }.map(|n| &mut n.val)
    }

    /// Map pointer to key-value reference pair
    #[inline]
    pub(crate) fn map_ptr(node: *mut Node<K, V>) -> Option<(&'a K, &'a mut V)> {
        unsafe { node.as_mut() }.map(|n| (&n.key, &mut n.val))
    }

    /// Map NonNull pointer to key-value reference pair
    #[inline]
    fn map_non_null(mut node: NonNull<Node<K, V>>) -> (&'a K, &'a mut V) {
        let n = unsafe { node.as_mut() };
        (&n.key, &mut n.val)
    }

    /// Insert a new node before the current one and return a reference to its value.
    ///
    /// If the list was empty, the cursor navigates to the inserted node.
    ///
    /// If the key matches the current node, the value of the current node is updated instead.
    pub fn insert_before(&mut self, key: K, val: V) -> &mut V {
        match unsafe { self.current.as_mut() } {
            Some(current) => {
                match self.parent.map.entry(key.clone()) {
                    Entry::Occupied(e) => Self::set_value(e, val), // Reuse node
                    Entry::Vacant(e) => {
                        let mut new = Node::insert(key, val, current.previous(), current);
                        e.insert(new.as_ptr());

                        if self.parent.list.head == current {
                            self.parent.list.head = new.as_ptr();
                        }

                        unsafe { &mut new.as_mut().val }
                    }
                }
            }
            None => self.set_only_node(key, val), // List is empty
        }
    }

    /// Insert node before the current one and return a reference to its value.
    ///
    /// The cursor navigates to the inserted node, if the list was empty.
    ///
    /// If the key matches the current node, the value of the current node is updated instead.
    pub fn insert_after(&mut self, key: K, val: V) -> &mut V {
        match unsafe { self.current.as_mut() } {
            Some(current) => {
                match self.parent.map.entry(key.clone()) {
                    Entry::Occupied(e) => Self::set_value(e, val), // Reuse node
                    Entry::Vacant(e) => {
                        let mut new = Node::insert(key, val, current, current.next());
                        e.insert(new.as_ptr());

                        if self.parent.list.tail == current {
                            self.parent.list.tail = new.as_ptr();
                        }

                        unsafe { &mut new.as_mut().val }
                    }
                }
            }
            None => self.set_only_node(key, val), // List is empty
        }
    }

    /// Set the only node in the list. Only call this, when list is empty.
    #[cold]
    fn set_only_node(&mut self, key: K, val: V) -> &mut V {
        let mut new = Node::insert(key.clone(), val, null_mut(), null_mut());
        let ptr = new.as_ptr();

        self.parent.list.head = ptr;
        self.parent.list.tail = ptr;
        self.current = ptr;

        self.parent.map.insert_unique_unchecked(key, ptr);

        unsafe { &mut new.as_mut().val }
    }

    /// Set the value of an occupied entry
    #[cold]
    fn set_value(mut entry: OccupiedEntry<'_, K, *mut Node<K, V>, S>, val: V) -> &mut V {
        unsafe {
            let ptr = *entry.get_mut();
            (*ptr).val = val;
            &mut (*ptr).val
        }
    }

    /// Remove the current node and return its key and value.
    /// Returns [None], if list is empty.
    ///
    /// Navigates the cursor to the previous node.
    /// If removed node was the head of the list, navigates it to the next node.
    /// If the list becomes empty, the cursor points to no node after the call.
    pub fn remove(&mut self) -> Option<(K, V)> {
        unsafe { self.current.as_mut() }.map(|current| {
            if self.parent.list.head == current {
                self.parent.list.head = current.next();
            }
            if self.parent.list.tail == current {
                self.parent.list.tail = current.previous();
            }
            if self.parent.saved == current {
                self.parent.saved = null_mut();
            }

            self.current = if !current.previous().is_null() {
                current.previous()
            } else {
                current.next()
            };

            current.remove();

            let current = unsafe { Box::from_raw(current) };
            (current.key, current.val)
        })
    }

    /// Remember the current cursor position for efficiently navigating to the this node later on using the
    /// `resume()` methods on [Cursor], [CursorMut] and [LinkedMap] or the `get_saved()` and `resume_mut()` methods on
    /// [LinkedMap].
    ///
    /// Note that the only operation that silently invalidates a saved position is removing the saved node. Inserting
    /// new modes anywhere in the list or changing the saved node's siblings does not.
    ///
    /// Only up to 1 node can be saved per [LinkedMap] at any given time.
    pub fn save(&mut self) {
        self.parent.saved = self.current;
    }

    /// Clear any saved node. See [CursorMut::save()](CursorMut::save) for details.
    pub fn clear_saved(&mut self) {
        self.parent.saved = null_mut();
    }

    // TODO: moving functions for current node: move_to_back, move_to_front, move_by(n, direction) -> int, move_to(n, count_direction))
}
