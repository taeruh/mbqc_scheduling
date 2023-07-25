use core::slice;
use std::{
    cmp::Ordering,
    iter::{
        self,
        Enumerate,
    },
    mem,
};

#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    Base,
    Full,
    Init,
    Iterable,
    IterableBase,
};
use crate::slice_extension::GetTwoMutSlice;

/// A newtype wrapper around [Vec].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BufferedVector<T> {
    /// The inner type of our (n)ewtype.
    pub n: Vec<T>,
}

impl<T> BufferedVector<T> {
    /// Creates a new empty [BufferedVector].
    pub fn new() -> Self {
        Self { n: Vec::new() }
    }

    /// Creates a new empty [BufferedVector] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { n: Vec::with_capacity(capacity) }
    }

    /// Wrap a [Vec] into a [BufferedVector].
    pub fn wrap(vec: Vec<T>) -> Self {
        Self { n: vec }
    }
}

impl<T> From<Vec<T>> for BufferedVector<T> {
    fn from(vec: Vec<T>) -> Self {
        Self { n: vec }
    }
}

impl<T> FromIterator<(usize, T)> for BufferedVector<T> {
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iter: I) -> Self {
        let mut res = Vec::new();
        for (key, value) in iter {
            res.insert(key, value);
        }
        Self { n: res }
    }
}

impl<'l, T> IntoIterator for &'l BufferedVector<T> {
    type Item = (usize, &'l T);
    type IntoIter = Enumerate<slice::Iter<'l, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.n.iter().enumerate()
    }
}

impl<'l, T> IntoIterator for &'l mut BufferedVector<T> {
    type Item = (usize, &'l mut T);
    type IntoIter = Enumerate<slice::IterMut<'l, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.n.iter_mut().enumerate()
    }
}

impl<T> IntoIterator for BufferedVector<T> {
    type Item = (usize, T);
    type IntoIter = Enumerate<<Vec<T> as IntoIterator>::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.n.into_iter().enumerate()
    }
}

/// Note that [BufferedVector] is essentially a [Vec]. Therefore, we can basically only
/// remove Pauli stacks at the end without screwing things up. When inserting Pauli
/// stacks at qubits above the length, buffer stacks are added.
impl<T> Base for BufferedVector<T>
where
    T: Clone + Default,
{
    type TB = T;
    fn insert(&mut self, key: usize, value: T) -> Option<T> {
        let len = self.len();
        match key.cmp(&len) {
            Ordering::Less => Some(mem::replace(
                self.get_mut(key)
                    .expect("can't be out of bounds in this match arm"),
                value,
            )),
            Ordering::Equal => {
                self.n.push(value);
                None
            }
            Ordering::Greater => {
                let diff = key - len;
                self.n.try_reserve(diff).unwrap_or_else(|e| {
                    panic!("error when trying to reserve enough memory: {e}")
                });
                self.n.extend(iter::repeat(T::default()).take(diff));
                self.n.push(value);
                None
            }
        }
    }

    fn remove(&mut self, key: usize) -> Option<T> {
        match key.cmp(&(self.len().checked_sub(1)?)) {
            Ordering::Less => panic!(
                "this type, which is basically a Vec, only allows removing elements \
                 consecutively from the end"
            ),
            Ordering::Equal => Some(
                self.n
                    .pop()
                    .expect("bug: we checked above that len is bigger than 0"),
            ),
            Ordering::Greater => None,
        }
    }

    #[inline(always)]
    fn get(&self, key: usize) -> Option<&T> {
        self.n.get(key)
    }

    #[inline(always)]
    fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        self.n.get_mut(key)
    }

    fn get_two_mut(&mut self, key_a: usize, key_b: usize) -> Option<(&mut T, &mut T)> {
        self.n.get_two_mut(key_a, key_b)
    }

    fn len(&self) -> usize {
        self.n.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.n.is_empty()
    }
}

impl<T> Iterable for BufferedVector<T>
where
    T: Default + Clone,
{
    type TI = T;
    type Iter<'l> = <&'l Self as IntoIterator>::IntoIter where T: 'l;
    type IterMut<'l> = <&'l mut Self as IntoIterator>::IntoIter where T: 'l;

    #[inline(always)]
    fn iter(&self) -> Self::Iter<'_> {
        self.into_iter()
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.into_iter()
    }
}

impl<T> Init for BufferedVector<T>
where
    T: Clone + Default,
{
    fn init(len: usize) -> Self {
        Self { n: vec![Default::default(); len] }
    }
}

impl<T> IterableBase for BufferedVector<T>
where
    T: Default + Clone,
{
    type T = T;
}
impl<T> Full for BufferedVector<T> where T: Default + Clone {}
