/*!
This module provides [Tracker]s that are similar to the ones in [frames](super::frames),
with the major difference that there's effectively only one frames, which adds up
multiple tracked Paulis.
*/

#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    unwrap_get_mut,
    unwrap_get_two_mut,
    MissingStack,
    PauliString,
    Tracker,
};
use crate::{
    collection::{
        Base,
        Iterable,
    },
    pauli::Pauli,
};

// todo: make it generic and also do it with a hashmap

/// An implementor of [Tracker], similar to [Frames](super::frames::Frames), with the
/// difference, that instead of storing each Pauli frame, it adds the Pauli frames (mod
/// 2).
// I'm not sure what the most efficient inner type would be here, Vec<bool>, Vec<Pauli>,
// BitVec, ...
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Live<S> {
    storage: S,
}

impl<S> From<S> for Live<S> {
    fn from(value: S) -> Self {
        Self { storage: value }
    }
}

impl<T> Live<T> {
    pub fn into(self) -> T {
        self.storage
    }
}

impl<S> AsRef<S> for Live<S> {
    fn as_ref(&self) -> &S {
        &self.storage
    }
}

impl<S, T> Live<S>
where
    S: Base<T = T>,
{
    /// Returns a mutable reference to an element at index. Returns [None] if out of
    /// bounds.
    pub fn get_mut(&mut self, bit: usize) -> Option<&mut T> {
        self.storage.get_mut(bit)
    }
}

impl<S, T> Live<S>
where
    S: Iterable<T = T>,
{
    /// Returns a reference to an element at index. Returns [None] if out of bounds.
    pub fn get(&self, bit: usize) -> Option<&T> {
        self.storage.get(bit)
    }
}

macro_rules! single {
    ($($name:ident,)*) => {$(
        fn $name(&mut self, bit: usize) {
            unwrap_get_mut!(self.storage, bit, stringify!($name)).$name()
        }
    )*};
}

macro_rules! movements {
    ($(($name:ident, $plus:ident, $set:ident),)*) => {$(
        fn $name(&mut self, source: usize, destination: usize) {
            let (s, d) =
                unwrap_get_two_mut!(self.storage, source, destination, stringify!($name));
            d.$plus(s);
            s.$set(false);
        }
    )*};
}

/// Note that the inner storage type is basically a vector. Therefore, the it may
/// contain buffer qubits, even though they were not explicitly initialized.
impl<S, P> Tracker for Live<S>
where
    S: Base<T = P>,
    P: Pauli + Clone,
{
    type Stack = P;
    type Pauli = P;

    movements!(
        (move_x_to_x, xpx, set_x),
        (move_x_to_z, zpx, set_x),
        (move_z_to_x, xpz, set_z),
        (move_z_to_z, zpz, set_z),
    );

    fn init(num_bits: usize) -> Self {
        Live { storage: S::init(num_bits, P::I) }
    }

    fn new_qubit(&mut self, bit: usize) -> Option<Self::Stack> {
        self.storage.insert(bit, P::I)
    }

    fn track_pauli(&mut self, bit: usize, pauli: Self::Pauli) {
        if let Some(p) = self.storage.get_mut(bit) {
            p.add(pauli)
        }
    }
    fn track_pauli_string(&mut self, string: PauliString<Self::Pauli>) {
        for (bit, pauli) in string {
            if let Some(p) = self.storage.get_mut(bit) {
                p.add(pauli)
            }
        }
    }

    single!(h, s,);

    fn cx(&mut self, control: usize, target: usize) {
        let (c, t) = unwrap_get_two_mut!(self.storage, control, target, "cx");
        t.xpx(c);
        c.zpz(t);
    }
    fn cz(&mut self, bit_a: usize, bit_b: usize) {
        let (a, b) = unwrap_get_two_mut!(self.storage, bit_a, bit_b, "cz");
        a.zpx(b);
        b.zpx(a);
    }

    fn measure(&mut self, bit: usize) -> Result<Self::Stack, MissingStack> {
        self.get_mut(bit).ok_or(MissingStack { bit }).cloned()
    }
}

#[cfg(test)]
mod tests {
    use coverage_helper::test;

    use super::*;
    use crate::{
        collection::BufferedVector,
        pauli::{
            PauliDense,
            PauliTuple,
        },
    };

    trait Pw: Pauli + Copy + Default + Into<PauliDense> + From<PauliDense> {}
    type Live<P> = super::Live<BufferedVector<P>>;

    mod single_actions {
        use super::*;
        use crate::tracker::test::impl_utils::{
            self,
            SingleAction,
            SingleResults,
            N_SINGLES,
        };

        type Action<P> = SingleAction<Live<P>>;

        #[cfg_attr(coverage_nightly, no_coverage)]
        fn runner<P: Pw>(action: Action<P>, result: SingleResults) {
            for (input, check) in (0u8..).zip(result.1) {
                let mut tracker = Live::<P>::init(2);
                tracker.track_pauli_string(impl_utils::single_init(input));
                (action)(&mut tracker, 0);
                assert_eq!(
                    P::into(*tracker.storage.get(0).unwrap()).storage(),
                    check,
                    "{}, {}",
                    result.0,
                    input
                );
            }
        }

        #[cfg_attr(coverage_nightly, no_coverage)]
        pub(super) fn run<P: Pw>() {
            let actions: [Action<P>; N_SINGLES] = [Live::h, Live::s];
            impl_utils::single_check(runner, actions);
        }
    }

    mod double_actions {
        use super::*;
        use crate::tracker::test::impl_utils::{
            self,
            DoubleAction,
            DoubleResults,
            N_DOUBLES,
        };

        type Action<P> = DoubleAction<Live<P>>;

        #[cfg_attr(coverage_nightly, no_coverage)]
        fn runner<P: Pw>(action: Action<P>, result: DoubleResults) {
            for (input, check) in (0u8..).zip(result.1) {
                let mut tracker = Live::init(2);
                tracker.track_pauli_string(impl_utils::double_init(input));
                (action)(&mut tracker, 0, 1);
                let output = impl_utils::double_output(tracker.storage.into_iter());
                assert_eq!(output, check, "{}, {}", result.0, input);
            }
        }

        pub(super) fn run<T: Pw>() {
            let actions: [Action<T>; N_DOUBLES] = [
                Live::cx,
                Live::cz,
                Live::move_x_to_x,
                Live::move_x_to_z,
                Live::move_z_to_x,
                Live::move_z_to_z,
            ];

            impl_utils::double_check(runner, actions);
        }
    }

    macro_rules! test_actions {
        ($(($pauli:ty, $module:ident),)*) => {$(
            mod $module {
                use super::test;
                #[rustfmt::skip]
                use super::{double_actions, single_actions, Pw, $pauli};
                impl Pw for $pauli {}
                #[test]
                fn single_actions() {
                    single_actions::run::<$pauli>();
                }
                #[test]
                fn double_actions() {
                    double_actions::run::<$pauli>();
                }
            }
        )*};
    }

    test_actions!((PauliDense, pauli_dense), (PauliTuple, pauli_tuple),);

    #[test]
    fn new_qubit_and_measure() {
        let mut tracker = Live::<PauliTuple>::init(1);
        tracker.track_x(0);
        assert_eq!(tracker.new_qubit(0), Some(PauliTuple::X));
        assert_eq!(tracker.new_qubit(1), None);
        tracker.track_y(0);
        assert_eq!(**tracker.as_ref(), vec![PauliTuple::Y, PauliTuple::I]);
        assert_eq!(tracker.measure(0), Ok(PauliTuple::Y));
        assert_eq!(tracker.new_qubit(3), None);
        // assert_eq!(
        //     *tracker.as_ref(),
        //     vec![
        //         PauliTuple::new_x(),
        //         PauliTuple::new_i(),
        //         PauliTuple::new_i(),
        //         PauliTuple::new_i()
        //     ]
        // );
    }

    //
}
