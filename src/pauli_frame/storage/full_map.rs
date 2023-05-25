use std::{
    collections::{
        hash_map,
        HashMap,
    },
    iter::Map,
};

use super::super::{
    PauliStorage,
    PauliVec,
};

pub type FullMap = HashMap<usize, PauliVec>;

impl PauliStorage for FullMap {
    type IterMut<'a> = Map<
        hash_map::IterMut<'a, usize, PauliVec>,
        fn((&usize, &'a mut PauliVec)) -> (usize, &'a mut PauliVec),
    >;
    type Iter<'a> = Map<
        hash_map::Iter<'a, usize, PauliVec>,
        fn((&usize, &'a PauliVec)) -> (usize, &'a PauliVec),
    >;

    fn insert_pauli(&mut self, qubit: usize, pauli: PauliVec) -> Option<PauliVec> {
        self.insert(qubit, pauli)
    }

    fn remove_pauli(&mut self, qubit: usize) -> Option<PauliVec> {
        self.remove(&qubit)
    }

    fn get(&self, qubit: usize) -> Option<&PauliVec> {
        self.get(&qubit)
    }

    fn get_mut(&mut self, qubit: usize) -> Option<&mut PauliVec> {
        self.get_mut(&qubit)
    }

    fn get_two_mut(
        &mut self,
        qubit_a: usize,
        qubit_b: usize,
    ) -> Option<(&mut PauliVec, &mut PauliVec)> {
        if qubit_a == qubit_b {
            return None;
        }
        // Safety: we checked above that the keys are different, so it is impossible
        // that we create two mutable references to the same object (except if there is
        // a bug in hashing algorithm)
        //
        // I do not know why this doesn't trigger an stack-borrow error in miri, but
        // doing basically the same with Vec/slice does trigger an error. In general it
        // would be cleaner to go over pointers as I do it for the MappedVector but a
        // HashMap is more complicated and the tools for that are not stable yet
        let a = unsafe { &mut *(self.get_mut(&qubit_a)? as *mut PauliVec) };
        let b = unsafe { &mut *(self.get_mut(&qubit_b)? as *mut PauliVec) };
        // that would catch a bug in the hashing algorithm
        // assert!(!std::ptr::eq(a, b));
        Some((a, b))
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter().map(|(&i, p)| (i, p))
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut().map(|(&i, p)| (i, p))
    }

    fn init(num_qubits: usize) -> Self {
        let mut ret = HashMap::with_capacity(num_qubits);
        for i in 0..num_qubits {
            ret.insert(i, PauliVec::new());
        }
        ret
    }
}
