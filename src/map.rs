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
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    /// Construct and navigate cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
    ///
    /// If no node is currently saved, returns [None].
    pub fn resume(&mut self) -> Option<CursorMut<'_, K, V, S>> {
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
}

// TODO: cursor constructors from head, tail and at key (get_cursor)
// TODO: get_saved() -> (&K, &V)
// TODO: get_saved_mut() -> (&K, &mut V)
// TODO: port as many methods and trait impls of the stdlib linked_list and hashbrown::HashMap as possible
