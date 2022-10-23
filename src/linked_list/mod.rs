mod node;
mod tests;

mod cursor;
pub use cursor::{Cursor, CursorMut};

use node::Node;
use std::ptr::null_mut;

/// Doubly-linked list with cursor iteration support
pub(crate) struct LinkedList<K, V> {
    /// First node of list. null, if list is empty.
    head: *mut Node<K, V>,

    /// Last node of the list. null, if list is empty.
    tail: *mut Node<K, V>,
}

impl<K, V> Drop for LinkedList<K, V> {
    fn drop(&mut self) {
        let mut next = self.head;
        while !next.is_null() {
            let b = unsafe { Box::from_raw(next) };
            next = b.next();
        }
    }
}

impl<K, V> LinkedList<K, V> {
    /// Create new empty list
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    // /// Return a forward mutable iterator over the list
    // pub fn iter_mut(
    //     &mut self,
    // ) -> impl ExactSizeIterator<Item = &'_ mut T> + FusedIterator<Item = &'_ mut T> {
    //     IterMut::<'_, T, Forward, N>::new(self.cursor_mut())
    // }

    // /// Return a backward mutable iterator over the list
    // pub fn iter_mut_reverse(
    //     &mut self,
    // ) -> impl ExactSizeIterator<Item = &'_ mut T> + FusedIterator<Item = &'_ mut T> {
    //     IterMut::<'_, T, Backward, N>::new({
    //         let mut c = self.cursor_mut();
    //         c.to_end();
    //         c
    //     })
    // }
}

// /// Advances a cursor in a direction
// trait Advance {
//     /// Try to advance the cursor in a direction and return, if it was
//     fn try_advance<'a, T, const N: usize>(c: &mut CursorMut<'a, T, N>) -> bool
//     where
//         T: Sized + 'static;
// }

// /// Advances the cursor forward
// struct Forward;

// impl Advance for Forward {
//     #[inline]
//     fn try_advance<'a, T, const N: usize>(c: &mut CursorMut<'a, T, N>) -> bool
//     where
//         T: Sized + 'static,
//     {
//         c.next()
//     }
// }

// /// Advance the cursor backward
// struct Backward;

// impl Advance for Backward {
//     #[inline]
//     fn try_advance<'a, T, const N: usize>(c: &mut CursorMut<'a, T, N>) -> bool
//     where
//         T: Sized + 'static,
//     {
//         c.previous()
//     }
// }

// /// Forward iterator for cursors
// struct IterMut<'a, T, A, const N: usize>
// where
//     T: Sized + 'static,
//     A: Advance,
// {
//     visited_first: bool,
//     cursor: CursorMut<'a, T, N>,
//     pd: PhantomData<A>,
// }

// impl<'a, T, A, const N: usize> IterMut<'a, T, A, N>
// where
//     T: Sized + 'static,
//     A: Advance,
// {
//     fn new(c: CursorMut<'a, T, N>) -> Self {
//         Self {
//             visited_first: false,
//             cursor: c,
//             pd: PhantomData,
//         }
//     }
// }

// impl<'a, T, A, const N: usize> Iterator for IterMut<'a, T, A, N>
// where
//     T: Sized + 'static,
//     A: Advance,
// {
//     type Item = &'a mut T;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         if !self.visited_first {
//             self.visited_first = true;
//         } else {
//             if !A::try_advance(&mut self.cursor) {
//                 return None;
//             }
//         }

//         self.cursor.value()
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.len(), Some(self.len()))
//     }
// }

// impl<'a, T, A, const N: usize> ExactSizeIterator for IterMut<'a, T, A, N>
// where
//     T: Sized + 'static,
//     A: Advance,
// {
//     #[inline]
//     fn len(&self) -> usize {
//         self.cursor.list.len()
//     }
// }

// impl<'a, T, A, const N: usize> FusedIterator for IterMut<'a, T, A, N>
// where
//     T: Sized + 'static,
//     A: Advance,
// {
// }

// impl<T, const N: usize> FromIterator<T> for LinkedList<T, N>
// where
//     T: Sized + 'static,
// {
//     fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
//         let mut ll = LinkedList::new();
//         let mut c = ll.cursor_mut();
//         for val in iter.into_iter() {
//             c.insert_after(val);
//             c.next();
//         }
//         ll
//     }
// }
