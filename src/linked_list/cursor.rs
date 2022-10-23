use hashbrown::{
    hash_map::{Entry, OccupiedEntry},
    HashMap,
};

use super::{node::Node, LinkedList};
use paste::paste;
use std::{
    hash::{BuildHasher, Hash},
    ptr::NonNull,
};

/// Implement functionality common to both mutable and immutable cursors
macro_rules! impl_common {
    ($key_value:ty, $iterator:ident) => {
        /// Helper for getting the current node
        #[inline]
        fn node(&self) -> Option<&mut Node<K, V>> {
            unsafe { self.node.as_mut() }
        }

        /// Navigate to the start of the linked list
        pub fn to_start(&mut self) {
            self.node = self.list.head;
        }

        /// Navigate to the end of the linked list
        pub fn to_end(&mut self) {
            self.node = self.list.tail;
        }

        /// Returns a reference to the current node's key in the map.
        /// Only returns None, if the current linked list is empty.
        pub fn key(&self) -> Option<&K> {
            self.node().map(|n| &n.key)
        }

        /// Return a reference to the current node's key-value pair.
        /// Only returns None, if the list is empty.
        pub fn key_value(&self) -> Option<$key_value> {
            Self::map_ptr(self.node)
        }

        /// Return the number of elements in the parent list
        pub fn len(&self) -> usize {
            self.map.len()
        }

        /// Return , if the parent list is empty
        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        /// Returns the number of elements the map can hold without reallocating
        pub fn capacity(&self) -> usize {
            self.map.capacity()
        }

        /// Try to advances cursor to the next node and return the key and value of that node
        #[inline]
        #[allow(clippy::should_implement_trait)]
        pub fn next(&mut self) -> Option<$key_value> {
            unsafe { self.node.as_ref() }.and_then(|node| {
                unsafe { node.next().as_mut() }.map(|next| {
                    self.node = next;
                    Self::map_non_null(next.into())
                })
            })
        }

        /// Try to move cursor to the next previous and return the key and value of that node
        #[inline]
        pub fn previous(&mut self) -> Option<$key_value> {
            unsafe { self.node.as_ref() }.and_then(|node| {
                unsafe { node.previous().as_mut() }.map(|prev| {
                    self.node = prev;
                    Self::map_non_null(prev.into())
                })
            })
        }

        /// Try to navigate to the given key.
        /// Returns [None], if no such key found.
        pub fn to_key(&mut self, k: &K) -> Option<$key_value> {
            self.map.get(k).map(|n| {
                let n = *n;
                self.node = n;
                Self::map_non_null(unsafe { NonNull::new_unchecked(n) })
            })
        }

        /// Get the key and value of the next node (if any), without advancing the cursor
        pub fn peek_next(&self) -> Option<$key_value> {
            self.node().and_then(|n| Self::map_ptr(n.next()))
        }

        /// Get the key and value of the previous node (if any), without advancing the cursor
        pub fn peek_previous(&self) -> Option<$key_value> {
            self.node().and_then(|n| Self::map_ptr(n.previous()))
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
    };
}

/// Enables safe linked list traversal
pub struct Cursor<'a, K, V, S> {
    /// Parent list
    list: &'a LinkedList<K, V>,

    // Parent map
    map: &'a HashMap<K, *mut Node<K, V>, S>,

    /// Node the cursor is currently at. Can be null.
    node: *mut Node<K, V>,
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
    /// `list` and `map` must reference the same pool of key-value pairs.
    /// `node` must belong to the parent list and map and be null, only if the parents are empty.
    #[inline]
    pub(super) unsafe fn new(
        list: &'a LinkedList<K, V>,
        map: &'a HashMap<K, *mut Node<K, V>, S>,
        node: *mut Node<K, V>,
    ) -> Self {
        Self { list, map, node }
    }

    /// Return a reference to the current node's value.
    /// Only returns None, if the list is empty.
    pub fn value(&self) -> Option<&V> {
        self.node().map(|n| &n.val)
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
            list: self.list,
            map: self.map,
            node: self.node,
        }
    }
}

/// Enables safe linked list traversal and mutation
pub struct CursorMut<'a, K, V, S> {
    /// Parent list
    list: &'a mut LinkedList<K, V>,

    // Parent map
    map: &'a mut HashMap<K, *mut Node<K, V>, S>,

    /// Node the cursor is currently at. Can be null.
    node: *mut Node<K, V>,
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
    /// `list` and `map` must reference the same pool of key-value pairs.
    /// `node` must belong to the parent list and map and be null, only if the parents are empty.
    #[inline]
    pub(super) unsafe fn new(
        list: &'a mut LinkedList<K, V>,
        map: &'a mut HashMap<K, *mut Node<K, V>, S>,
        node: *mut Node<K, V>,
    ) -> Self {
        Self { list, map, node }
    }

    /// Return a reference to the current node's value.
    /// Only returns None, if the list is empty.
    pub fn value(&mut self) -> Option<&mut V> {
        unsafe { self.node.as_mut() }.map(|n| &mut n.val)
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
    pub fn insert_before(&mut self, k: K, v: V) -> &mut V {
        self.insert(k, v, |list, current, new| {
            new.set_previous(current.previous());
            new.set_next(current);

            if list.head == current {
                list.head = new;
            }
        })
    }

    /// Insert node before the current one and return a reference to its value.
    ///
    /// The cursor navigates to the inserted node, if the list was empty.
    ///
    /// If the key matches the current node, the value of the current node is updated instead.
    pub fn insert_after(&mut self, k: K, v: V) -> &mut V {
        self.insert(k, v, |list, current, new| {
            new.set_previous(current);
            new.set_next(current.next());

            if list.tail == current {
                list.tail = new;
            }
        })
    }

    /// Common logic for node insertion.
    ///
    /// `after_insert` accepts the parent list, the current node and the inserted node as arguments.
    #[inline(always)]
    fn insert(
        &mut self,
        k: K,
        v: V,
        after_insert: impl Fn(&mut LinkedList<K, V>, &mut Node<K, V>, &mut Node<K, V>),
    ) -> &mut V {
        match unsafe { self.node.as_mut() } {
            Some(current) => {
                match self.map.entry(k.clone()) {
                    Entry::Occupied(e) => Self::set_value(e, v), // Reuse node
                    Entry::Vacant(e) => {
                        let node = Node::new(k, v);
                        e.insert(node);

                        after_insert(self.list, current, unsafe { &mut *node });

                        unsafe { &mut (*node).val }
                    }
                }
            }
            None => self.set_only_node(k, v), // List is empty
        }
    }

    /// Set the only node in the list. Only call this, when list is empty.
    #[cold]
    fn set_only_node(&mut self, k: K, v: V) -> &mut V {
        let node = Node::new(k.clone(), v);

        self.list.head = node;
        self.list.tail = node;
        self.node = node;

        self.map.insert_unique_unchecked(k, node);

        unsafe { &mut (*node).val }
    }

    /// Set the value of an occupied entry
    #[cold]
    fn set_value(mut e: OccupiedEntry<'_, K, *mut Node<K, V>, S>, v: V) -> &mut V {
        unsafe {
            let ptr = *e.get_mut();
            (*ptr).val = v;
            &mut (*ptr).val
        }
    }

    /// Remove current node and return its key and value.
    /// Returns [None], if list is empty.
    ///
    /// Navigates the cursor to the previous node.
    /// If removed node was the head of the list, navigates it to the next node.
    /// If the list becomes empty, the cursor points to no node after the call.
    pub fn remove(&mut self) -> Option<(K, V)> {
        if self.node.is_null() {
            None
        } else {
            let current = unsafe { &mut *self.node };

            if self.list.head == current {
                self.list.head = current.next();
            }
            if self.list.tail == current {
                self.list.tail = current.previous();
            }

            self.node = if !current.previous().is_null() {
                current.previous()
            } else {
                current.next()
            };

            current.unlink();

            let current = unsafe { Box::from_raw(self.node) };
            Some((current.key, current.val))
        }
    }
}
