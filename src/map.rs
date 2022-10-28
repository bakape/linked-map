use std::{
    hash::{BuildHasher, Hash},
    ptr::null_mut,
};

use hashbrown::HashMap;

use crate::{
    linked_list::{node::Node, LinkedList},
    Cursor, CursorMut,
};

/// Key-value store with linked-list reordering capabilities a cursor API and memory
pub struct LinkedMap<K, V, S> {
    /// Stores node order
    pub(crate) list: LinkedList<K, V>,

    /// Stores key-value relations for quick lookup
    pub(crate) map: HashMap<K, *mut Node<K, V>, S>,

    /// A node stored by the user for reconstructing a cursor later on. Can be null.
    pub(crate) saved: *mut Node<K, V>,
}

impl<K, V, S> LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
{
    /// Create a new empty [LinkedMap]
    pub fn new() -> Self
    where
        S: Default,
    {
        Default::default()
    }
}

impl<K, V, S> LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    /// Construct and navigate cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
    ///
    /// If no node is currently saved, returns [None].
    pub fn resume(&self) -> Option<Cursor<'_, K, V, S>> {
        unsafe { self.saved.as_mut().map(|saved| Cursor::new(self, saved)) }
    }

    /// Construct and navigate a mutable cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
    ///
    /// If no node is currently saved, returns [None].
    pub fn resume_mut(&mut self) -> Option<CursorMut<'_, K, V, S>> {
        unsafe { self.saved.as_mut().map(|saved| CursorMut::new(self, saved)) }
    }

    /// Clear any saved node. See [CursorMut::save()](CursorMut::save) for details.
    pub fn clear_saved(&mut self) {
        self.saved = null_mut();
    }

    /// Iterate the list from head to tail
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        unsafe { Cursor::new(self, self.list.head) }.iter()
    }

    /// Iterate the list from tail to head
    pub fn iter_rev(&self) -> impl Iterator<Item = (&K, &V)> {
        unsafe { Cursor::new(self, self.list.tail) }.iter_rev()
    }

    /// Iterate the list mutably from head to tail
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        unsafe { CursorMut::new(self, self.list.head) }.iter()
    }

    /// Iterate the list mutably from tail to head
    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        unsafe { CursorMut::new(self, self.list.tail) }.iter_rev()
    }

    /// Create a cursor over the linked map.
    ///
    /// The Cursor is set to the start of the list.
    pub fn cursor(&self) -> Cursor<'_, K, V, S> {
        unsafe { Cursor::new(self, self.list.head) }
    }

    /// Create a mutable cursor over the linked map.
    ///
    /// The Cursor is set to the start of the list.
    pub fn cursor_mut(&mut self) -> CursorMut<'_, K, V, S> {
        unsafe { CursorMut::new(self, self.list.head) }
    }

    /// Create a cursor navigated to the passed key.
    ///
    /// This is a shorthand for constricting a cursor and calling `to_key()`.
    ///
    /// Returns [None], if the key is not in the map,
    pub fn cursor_at(&self, k: &K) -> Option<Cursor<K, V, S>> {
        self.map.get(k).map(|n| unsafe { Cursor::new(self, *n) })
    }

    /// Create a mutable cursor navigated to the passed key.
    ///
    /// This is a shorthand for constricting a cursor and calling `to_key()`.
    ///
    /// Returns [None], if the key is not in the map,
    pub fn cursor_at_mut(&mut self, k: &K) -> Option<CursorMut<K, V, S>> {
        self.map
            .get(k)
            .copied()
            .map(|n| unsafe { CursorMut::new(self, n) })
    }
}

impl<K, V, S> Default for LinkedMap<K, V, S>
where
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self {
            list: LinkedList::new(),
            map: Default::default(),
            saved: null_mut(),
        }
    }
}

// TODO: port as many methods and trait impls of the stdlib linked_list and hashbrown::HashMap as possible
// TODO: sort_by and sort_by_stable
// TODO: append and prepend instead of insert

impl<K, V, S> Clone for LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: Clone + 'static,
    S: BuildHasher + Default,
{
    fn clone(&self) -> Self {
        self.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

impl<K, V, S> FromIterator<(K, V)> for LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut lm = LinkedMap {
            list: LinkedList::new(),
            map: HashMap::with_capacity_and_hasher(iter.size_hint().0, S::default()),
            saved: null_mut(),
        };

        for (k, v) in iter {
            let n = lm.list.append(k.clone(), v);
            lm.map.insert(k, n.as_ptr());
        }

        lm
    }
}
