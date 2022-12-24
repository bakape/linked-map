use hashbrown::hash_map::Entry;

use crate::LinkedMap;

use paste::paste;
use std::{
    hash::{BuildHasher, Hash},
    ptr::{null_mut, NonNull},
};

use super::list::Node;

// TODO: add examples to all of the public ones

/// Implement functionality common to both mutable and immutable cursors
macro_rules! impl_common {
    ($key_value:ty, $iterator:ident) => {
        /// Helper for getting a reference to the current node
        #[inline]
        fn current(&self) -> Option<&mut Node<K, V>> {
            unsafe { self.current.as_mut() }
        }

        /// Navigate to the start of the linked list
        pub fn to_front(&mut self) {
            self.current = self.parent.list.head();
        }

        /// Navigate to the end of the linked list
        pub fn to_back(&mut self) {
            self.current = self.parent.list.tail();
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

        /// Try to navigate to the given key and return its key-value pair.
        /// Returns [None], if no such key found.
        pub fn to_key(&mut self, key: &K) -> Option<$key_value> {
            self.parent.map.get(key).map(|n| {
                self.current = n.as_ptr();
                Self::map_non_null(*n)
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

        /// Navigate cursor to a saved node position, saved via [CursorMut::save](CursorMut::save) and return it's
        /// key-value pair.
        ///
        /// If no node is currently saved, returns [None].
        pub fn resume(&mut self) -> Option<$key_value> {
            unsafe { self.parent.saved.as_mut() }.map(|n| {
                self.current = n;
                Self::map_non_null(n.into())
            })
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
    fn map_ptr(node: *mut Node<K, V>) -> Option<(&'a K, &'a V)> {
        unsafe { node.as_ref() }.map(|n| (&n.key, &n.val))
    }

    /// Map [NonNull] pointer to key-value reference pair
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
    #[inline]
    pub fn value(&mut self) -> Option<&mut V> {
        self.current().map(|n| &mut n.val)
    }

    /// Map pointer to key-value reference pair
    #[inline]
    fn map_ptr(node: *mut Node<K, V>) -> Option<(&'a K, &'a mut V)> {
        unsafe { node.as_mut() }.map(|n| (&n.key, &mut n.val))
    }

    /// Map [NonNull] pointer to key-value reference pair
    #[inline]
    fn map_non_null(mut node: NonNull<Node<K, V>>) -> (&'a K, &'a mut V) {
        let n = unsafe { node.as_mut() };
        (&n.key, &mut n.val)
    }

    /// Insert a new node before the current one.
    ///
    /// If the list was empty, the cursor navigates to the inserted node.
    ///
    /// If the key matches the current node, the value of the current node is updated instead.
    pub fn insert_before(&mut self, key: K, val: V) {
        match self.current() {
            Some(current) => {
                if current.key == key {
                    current.val = val;
                    return;
                }

                match self.parent.map.entry(key.clone()) {
                    Entry::Occupied(e) => unsafe {
                        self.parent.list.insert_before(*e.get_mut(), current.into());
                    },
                    Entry::Vacant(e) => {
                        let mut new = Node::new(key, val);
                        unsafe {
                            self.parent.list.insert_before(new, current.into());
                        }
                        e.insert(new);
                    }
                }
            }
            None => self.set_only_node(key, val), // List is empty
        }
    }

    /// Insert a new node after the current one.
    ///
    /// If the list was empty, the cursor navigates to the inserted node.
    ///
    /// If the key matches the current node, the value of the current node is updated instead.
    pub fn insert_after(&mut self, key: K, val: V) {
        match unsafe { self.current.as_mut() } {
            Some(current) => {
                if current.key == key {
                    current.val = val;
                    return;
                }

                match self.parent.map.entry(key.clone()) {
                    Entry::Occupied(e) => unsafe {
                        self.parent.list.insert_after(*e.get_mut(), current.into());
                    },
                    Entry::Vacant(e) => {
                        let mut new = Node::new(key, val);
                        unsafe {
                            self.parent.list.insert_after(new, current.into());
                        }
                        e.insert(new);
                    }
                };
            }
            None => self.set_only_node(key, val), // List is empty
        }
    }

    /// Set the only node in the list. Only call this, when list is empty.
    #[cold]
    fn set_only_node(&mut self, key: K, val: V) {
        let new = self.parent.list.append(key.clone(), val);
        self.parent.map.insert(key, new);
        self.current = new.as_ptr();
    }

    /// Remove the current node and return its key and value.
    /// Returns [None], if list is empty.
    ///
    /// Navigates the cursor to the previous node.
    /// If removed node was the head of the list, navigates it to the next node.
    /// If the list becomes empty, the cursor points to no node after the call.
    pub fn remove(&mut self) -> Option<(K, V)> {
        unsafe { self.current.as_mut() }.map(|current| {
            let navigate_to = if !current.previous().is_null() {
                current.previous()
            } else {
                current.next()
            };

            self.parent.list.remove(current.into());
            if self.parent.saved == current {
                self.parent.saved = null_mut();
            }

            let current = unsafe { Box::from_raw(current) };
            self.current = navigate_to;
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
    /// Only up to 1 node can be saved on a  [LinkedMap] at any given time.
    #[inline]
    pub fn save(&mut self) {
        self.parent.saved = self.current;
    }

    /// Clear any saved node. See [CursorMut::save()](CursorMut::save) for details.
    #[inline]
    pub fn clear_saved(&mut self) {
        self.parent.clear_saved()
    }

    /// Move the current node to the front of the list
    pub fn move_to_front(&mut self) {
        if let Some(current) = self.current() {
            self.parent.list.move_to_front(current.into());
        }
    }

    /// Move the current node to the back of the list
    pub fn move_to_back(&mut self) {
        if let Some(current) = self.current() {
            self.parent.list.move_to_back(current.into());
        }
    }
}
