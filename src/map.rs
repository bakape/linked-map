use std::{
    hash::{BuildHasher, Hash},
    ptr::null_mut,
};

use hashbrown::HashMap;

use crate::{
    linked_list::{node::Node, LinkedList},
    CursorMut,
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
}

// TODO: get_saved() -> (&K, &V)
// TODO: get_saved_mut() -> (&K, &mut V)
// TODO: port as many methods and trait impls of the stdlib linked_list and hashbrown::HashMap as possible
