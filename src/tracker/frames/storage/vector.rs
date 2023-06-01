use std::{
    cmp::Ordering,
    iter::Enumerate,
    ops::{
        Deref,
        DerefMut,
    },
    slice,
};

#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

use super::super::{
    PauliVec,
    StackStorage,
};
use crate::slice_extension::GetTwoMutSlice;

/// Basically a vector of [PauliVec]s. Restricted, but if that is no problem, and the
/// type is used correctly, it is more efficient than [Map](super::map::Map).
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector {
    frames: Vec<PauliVec>,
}

impl Deref for Vector {
    type Target = Vec<PauliVec>;
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Vector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frames
    }
}

impl IntoIterator for Vector {
    type Item = (usize, PauliVec);

    type IntoIter = Enumerate<<Vec<PauliVec> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.frames.into_iter().enumerate()
    }
}

impl StackStorage for Vector {
    type IterMut<'a> = Enumerate<slice::IterMut<'a, PauliVec>>
    where
        Self: 'a;
    type Iter<'a> = Enumerate<slice::Iter<'a, PauliVec>>
    where
        Self: 'a;

    fn insert_pauli(&mut self, bit: usize, pauli: PauliVec) -> Option<PauliVec> {
        match bit.cmp(&self.len()) {
            Ordering::Less => Some(pauli),
            Ordering::Equal => {
                self.push(pauli);
                None
            }
            Ordering::Greater => panic!(
                "this type, FixedVector, only allows consecutively inserting elements"
            ),
        }
    }

    fn remove_pauli(&mut self, bit: usize) -> Option<PauliVec> {
        match bit.cmp(&(self.len().checked_sub(1)?)) {
            Ordering::Less => panic!(
                "this type, FixedVector, only allows consecutively removing elements"
            ),
            Ordering::Equal => {
                Some(self.pop().expect("that's an implementation bug; please report"))
            }
            Ordering::Greater => None,
        }
    }

    #[inline(always)]
    fn get(&self, qubit: usize) -> Option<&PauliVec> {
        self.frames.get(qubit)
    }

    #[inline(always)]
    fn get_mut(&mut self, qubit: usize) -> Option<&mut PauliVec> {
        self.frames.get_mut(qubit)
    }

    fn get_two_mut(
        &mut self,
        qubit_a: usize,
        qubit_b: usize,
    ) -> Option<(&mut PauliVec, &mut PauliVec)> {
        self.frames.get_two_mut(qubit_a, qubit_b)
    }

    #[inline(always)]
    fn iter(&self) -> Self::Iter<'_> {
        self.frames.iter().enumerate()
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.frames.iter_mut().enumerate()
    }

    #[inline(always)]
    fn init(num_qubits: usize) -> Self {
        Self {
            frames: vec![PauliVec::new(); num_qubits],
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}
