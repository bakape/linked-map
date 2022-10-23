use crate::{Cursor, CursorMut};
use std::hash::{BuildHasher, Hash};
use std::iter::{FusedIterator, Iterator};

macro_rules! impl_iter {
    ($name:ident, $cursor:ident, $item:ty) => {
        macro_rules! impl_iter_direction {
            ($direction:ident, $method:ident) => {
                paste::paste! {
                    /// An iterator over the list's key-value pairs
                    pub struct [<$name $direction>]<'a, K, V, S>
                    where
                        K: Eq + Hash,
                        S: BuildHasher,
                    {
                        cursor: $cursor<'a, K, V, S>,
                    }

                    impl<'a, K, V, S> [<$name $direction>]<'a, K, V, S>
                    where
                        K: Eq + Hash + Clone + 'static,
                        V: 'static,
                        S: BuildHasher,
                    {
                        pub fn new(cursor: $cursor<'a, K, V, S>) -> Self {
                            Self{ cursor }
                        }
                    }

                    impl<'a, K, V, S> Iterator for [<$name $direction>]<'a, K, V, S>
                    where
                        K: Eq + Hash + Clone + 'static,
                        V: 'static,
                        S: BuildHasher,
                    {
                        type Item = $item;

                        #[inline]
                        fn next(&mut self) -> Option<Self::Item> {
                            self.cursor.$method()
                        }

                        #[inline]
                        fn size_hint(&self) -> (usize, Option<usize>) {
                            // Tracking the position of the cursor in the list just incase it is collected is too
                            // expensive.
                            // This prevents reallocations, if the entire list is collected, and that's good enough.
                            (0, self.cursor.len().into())
                        }
                    }

                    impl<'a, K, V, S> FusedIterator for [<$name $direction>]<'a, K, V, S>
                    where
                        K: Eq + Hash + Clone + 'static,
                        V: 'static,
                        S: BuildHasher,
                    {}
                }
            };
        }

        impl_iter_direction!(Forward, next);
        impl_iter_direction!(Backward, previous);
    };
}

impl_iter!(Iter, Cursor, (&'a K, &'a V));
impl_iter!(IterMut, CursorMut, (&'a K, &'a mut V));
