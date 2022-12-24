//! Parts of the map API are heavily inspired by [hashbrown](https://github.com/rust-lang/hashbrown).
//! Credit for those goes to the appropriate code authors.

use std::{
    hash::{BuildHasher, Hash},
    ptr::{null_mut, NonNull},
};

use hashbrown::{hash_map::Entry, HashMap};

pub use hashbrown::hash_map::DefaultHashBuilder;

use crate::{
    linked_list::{list::Node, LinkedList},
    Cursor, CursorMut,
};

/// Key-value store with linked-list reordering capabilities a cursor API and memory
pub struct LinkedMap<K, V, S = DefaultHashBuilder> {
    /// Stores node order
    pub(crate) list: LinkedList<K, V>,

    /// Stores key-value relations for quick lookup
    pub(crate) map: HashMap<K, NonNull<Node<K, V>>, S>,

    /// A node stored by the user for reconstructing a cursor later on. Can be null.
    pub(crate) saved: *mut Node<K, V>,
}

impl<K, V> LinkedMap<K, V, DefaultHashBuilder> {
    /// Create a new empty [LinkedMap]
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # HashDoS resistance
    ///
    /// The `hash_builder` normally use a fixed key by default and that does
    /// not allow the `LinkedMap` to be protected against attacks such as [`HashDoS`].
    /// Users who require HashDoS resistance should explicitly use
    /// [`ahash::RandomState`] or [`std::collections::hash_map::RandomState`]
    /// as the hasher when creating a [`LinkedMap`], for example with
    /// [`with_hasher`](LinkedMap::with_hasher) method.
    ///
    /// [`HashDoS`]: https://en.wikipedia.org/wiki/Collision_attack
    /// [`std::collections::hash_map::RandomState`]:
    /// https://doc.rust-lang.org/std/collections/hash_map/struct.RandomState.html
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let mut map: LinkedMap<&str, i32> = LinkedMap::new();
    /// assert_eq!(map.len(), 0);
    /// assert_eq!(map.capacity(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates an empty `LinkedMap` with the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    ///
    /// # HashDoS resistance
    ///
    /// The `hash_builder` normally use a fixed key by default and that does
    /// not allow the `LinkedMap` to be protected against attacks such as [`HashDoS`].
    /// Users who require HashDoS resistance should explicitly use
    /// [`ahash::RandomState`] or [`std::collections::hash_map::RandomState`]
    /// as the hasher when creating a [`LinkedMap`], for example with
    /// [`with_capacity_and_hasher`](LinkedMap::with_capacity_and_hasher) method.
    ///
    /// [`HashDoS`]: https://en.wikipedia.org/wiki/Collision_attack
    /// [`std::collections::hash_map::RandomState`]:
    /// https://doc.rust-lang.org/std/collections/hash_map/struct.RandomState.html
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let mut map: LinkedMap<&str, i32> = LinkedMap::with_capacity(10);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, Default::default()),
            ..Default::default()
        }
    }
}

impl<K, V, S> LinkedMap<K, V, S> {
    /// Creates an empty `LinkedMap` which will use the given hash builder to hash
    /// keys.
    ///
    /// The map is initially created with a capacity of 0, so it will not
    /// allocate until it is first inserted into.
    ///
    /// # HashDoS resistance
    ///
    /// The `hash_builder` normally use a fixed key by default and that does
    /// not allow the `LinkedMap` to be protected against attacks such as [`HashDoS`].
    /// Users who require HashDoS resistance should explicitly use
    /// [`ahash::RandomState`] or [`std::collections::hash_map::RandomState`]
    /// as the hasher when creating a [`LinkedMap`].
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the LinkedMap to be useful, see its documentation for details.
    ///
    /// [`HashDoS`]: https://en.wikipedia.org/wiki/Collision_attack
    /// [`std::collections::hash_map::RandomState`]: https://doc.rust-lang.org/std/collections/hash_map/struct.RandomState.html
    /// [`BuildHasher`]: https://doc.rust-lang.org/std/hash/trait.BuildHasher.html
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::{LinkedMap, DefaultHashBuilder};
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut map = LinkedMap::with_hasher(s);
    /// assert_eq!(map.len(), 0);
    /// assert_eq!(map.capacity(), 0);
    ///
    /// map.append(1, 2);
    /// ```
    #[inline]
    pub const fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: HashMap::with_hasher(hash_builder),
            list: LinkedList::new(),
            saved: null_mut(),
        }
    }

    /// Creates an empty `LinkedMap` with the specified capacity, using `hash_builder`
    /// to hash the keys.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    ///
    /// # HashDoS resistance
    ///
    /// The `hash_builder` normally use a fixed key by default and that does
    /// not allow the `LinkedMap` to be protected against attacks such as [`HashDoS`].
    /// Users who require HashDoS resistance should explicitly use
    /// [`ahash::RandomState`] or [`std::collections::hash_map::RandomState`]
    /// as the hasher when creating a [`LinkedMap`].
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the LinkedMap to be useful, see its documentation for details.
    ///
    /// [`HashDoS`]: https://en.wikipedia.org/wiki/Collision_attack
    /// [`std::collections::hash_map::RandomState`]: https://doc.rust-lang.org/std/collections/hash_map/struct.RandomState.html
    /// [`BuildHasher`]: https://doc.rust-lang.org/std/hash/trait.BuildHasher.html
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::{LinkedMap, DefaultHashBuilder};
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut map = LinkedMap::with_capacity_and_hasher(10, s);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 10);
    ///
    /// map.append(1, 2);
    /// ```
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hash_builder),
            list: LinkedList::new(),
            saved: null_mut(),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let mut a = LinkedMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.append(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let mut a = LinkedMap::new();
    /// assert!(a.is_empty());
    /// a.append(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the map might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let map: LinkedMap<i32, i32> = LinkedMap::with_capacity(100);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

// TODO: add examples to all of these
impl<K, V, S> LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    /// Inserts a key-value pair at the start of the [LinkedMap].
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [`std::collections`]
    /// [module-level documentation] for more.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    /// [`std::collections`]: https://doc.rust-lang.org/std/collections/index.html
    /// [module-level documentation]: https://doc.rust-lang.org/std/collections/index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_map::LinkedMap;
    ///
    /// let mut map = LinkedMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn prepend(&mut self, k: K, mut v: V) -> Option<V> {
        match self.map.entry(k.clone()) {
            Entry::Occupied(mut e) => {
                let node = e.get_mut();
                std::mem::swap(&mut unsafe { node.as_mut() }.val, &mut v);
                unsafe { CursorMut::new(self, node.as_ptr()) }.move_to_front(NavigateTo::MovedNode);
                Some(v)
            }
            Entry::Vacant(e) => {
                e.insert(self.list.prepend(k, v));
                None
            }
        }
    }

    /// Inserts a key-value pair at the end of the [LinkedMap].
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [`std::collections`]
    /// [module-level documentation] for more.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    /// [`std::collections`]: https://doc.rust-lang.org/std/collections/index.html
    /// [module-level documentation]: https://doc.rust-lang.org/std/collections/index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn append(&mut self, k: K, v: V) -> Option<V> {
        todo!()
    }
}

// TODO: add examples to all of these
impl<K, V, S> LinkedMap<K, V, S>
where
    K: Eq + Hash + Clone + 'static,
    V: 'static,
    S: BuildHasher,
{
    /// Construct and navigate cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
    ///
    /// If no node is currently saved, returns [None].
    #[inline]
    pub fn resume(&self) -> Option<Cursor<'_, K, V, S>> {
        unsafe { self.saved.as_mut().map(|saved| Cursor::new(self, saved)) }
    }

    /// Construct and navigate a mutable cursor to a saved node position, saved via [CursorMut::save](CursorMut::save).
    ///
    /// If no node is currently saved, returns [None].
    #[inline]
    pub fn resume_mut(&mut self) -> Option<CursorMut<'_, K, V, S>> {
        unsafe { self.saved.as_mut().map(|saved| CursorMut::new(self, saved)) }
    }

    /// Clear any saved node. See [CursorMut::save()](CursorMut::save) for details.
    #[inline]
    pub fn clear_saved(&mut self) {
        self.saved = null_mut();
    }

    /// Iterate the list from head to tail
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        unsafe { Cursor::new(self, self.list.head()) }.iter()
    }

    /// Iterate the list from tail to head
    #[inline]
    pub fn iter_rev(&self) -> impl Iterator<Item = (&K, &V)> {
        unsafe { Cursor::new(self, self.list.tail()) }.iter_rev()
    }

    /// Iterate the list mutably from head to tail
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        unsafe { CursorMut::new(self, self.list.head()) }.iter()
    }

    /// Iterate the list mutably from tail to head
    #[inline]
    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        unsafe { CursorMut::new(self, self.list.tail()) }.iter_rev()
    }

    /// Create a cursor over the linked map.
    ///
    /// The Cursor is set to the start of the list.
    #[inline]
    pub fn cursor(&self) -> Cursor<'_, K, V, S> {
        unsafe { Cursor::new(self, self.list.head()) }
    }

    /// Create a mutable cursor over the linked map.
    ///
    /// The Cursor is set to the start of the list.
    #[inline]
    pub fn cursor_mut(&mut self) -> CursorMut<'_, K, V, S> {
        unsafe { CursorMut::new(self, self.list.head()) }
    }

    /// Create a cursor navigated to the passed key.
    ///
    /// This is a shorthand for constricting a cursor and calling `to_key()`.
    ///
    /// Returns [None], if the key is not in the map,
    #[inline]
    pub fn cursor_at(&self, k: &K) -> Option<Cursor<K, V, S>> {
        self.map
            .get(k)
            .map(|n| unsafe { Cursor::new(self, n.as_ptr()) })
    }

    /// Create a mutable cursor navigated to the passed key.
    ///
    /// This is a shorthand for constricting a cursor and calling `to_key()`.
    ///
    /// Returns [None], if the key is not in the map,
    #[inline]
    pub fn cursor_at_mut(&mut self, k: &K) -> Option<CursorMut<K, V, S>> {
        self.map
            .get(k)
            .copied()
            .map(|n| unsafe { CursorMut::new(self, n.as_ptr()) })
    }
}

impl<K, V, S> Default for LinkedMap<K, V, S>
where
    S: Default,
{
    #[inline]
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
    K: Eq + Hash + Clone + 'static,
    V: 'static,
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
            lm.append(k, v);
        }

        lm
    }
}
