// #![cfg(feature = "circuit")]

use std::{fmt::Debug, hash::BuildHasherDefault};

use hashbrown::HashMap;
use pauli_tracker::{
    // boolean_vector::BooleanVector,
    circuit::{
        CliffordCircuit,
        DummyCircuit,
        RandomMeasurementCircuit,
        TrackedCircuit,
    },
    collection::{
        BufferedVector,
        Init,
        Iterable,
        Map,
    },
    pauli::{
        Pauli,
        PauliStack,
        PauliTuple,
    },
    tracker::{
        frames::{
            induced_order::{
                self,
                // PartialOrderGraph,
            },
            Frames,
        },
        live, Tracker,
    },
};
use proptest::{
    arbitrary::any,
    prop_oneof, proptest,
    strategy::Strategy,
    test_runner::{Config, FileFailurePersistence},
};
use rustc_hash::FxHasher;

// type BoolVec = bitvec::vec::BitVec;
// type BoolVec = pauli_tracker::boolean_vector::bitvec_simd::SimdBitVec;
pub type BoolVec = bit_vec::BitVec;

pub type Storage = Map<PauliStack<BoolVec>, BuildHasherDefault<FxHasher>>;
type Live<P> = live::Live<BufferedVector<P>>;

// const MAX_INIT: usize = 100;
// const MAX_OPS: usize = 1000;
// const MAX_INIT: usize = 2;
// const MAX_OPS: usize = 10;
// proptest! {
//     #![proptest_config(Config {
//         cases: 1,
//         // cases: 10,
//         // proptest! just overwrites this (see source code); it doesn't really matter,
//         // except that we get a warning but that is ok; we could solve it by manually
//         // doing what proptest! does (the basics are straightforward, but it also does
//         // some nice things that are not straightforward)
//         failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
//             "regressions",
//         ))),
//         ..Default::default()
//     })]
//     #[test]
//     #[ignore = "run proptests explicitly"]
//     fn proptest(init in (0..MAX_INIT), ops in vec_operation(MAX_OPS)) {
//         roundtrip(init, ops);
//     }
// }

// // given some operations, we perform the pauli tracking with Frames and create the
// // dependency graph. This graph is checked whether it doesn't promise something wrong
// // and whether it is optimal. Then we also track Paulis via LiveVector and check
// // whether the results are compatible with results from Frames
// fn roundtrip(init: usize, ops: Vec<Operation>) {
//     // println!("len:  {}", ops.len());
//     // println!("init: {}", init);
//     let mut generator = Instructor::new(init, ops);
//     let mut circuit = TrackedCircuit {
//         circuit: DummyCircuit {},
//         tracker: Frames::<Storage>::init(init),
//         storage: Storage::default(),
//     };
//     let mut measurements = WhereMeasured(Vec::new());
//     generator.apply(&mut circuit, &mut measurements);
//     circuit.tracker.measure_and_store_all(&mut circuit.storage);

//     if !measurements.0.is_empty() {
//         let graph = induced_order::get_order(
//             <Storage as Iterable>::iter_pairs(&circuit.storage),
//             &measurements.0,
//         );
//         check_graph(&graph, &circuit.storage, &measurements.0).unwrap();
//     }

//     // println!("graph: {:?}", graph);
//     // println!("graph.len: {}", graph.len());
//     // println!("{:?}", measurements.0);

//     // println!("{:?}", generator.operations);
//     // println!("{:?}\n", storage::sort_by_bit(&circuit.storage));

//     generator.reinit(init);
//     let mut live_circuit = TrackedCircuit {
//         circuit: RandomMeasurementCircuit {},
//         tracker: Live::init(init),
//         storage: (),
//     };
//     let mut measurements = ResultMeasured(Vec::new());
//     generator.apply(&mut live_circuit, &mut measurements);
//     // println!("{:?}", measurements);
//     // println!("{:?}", live_circuit.tracker);

//     let mut check = vec![PauliTuple::new_i(); generator.used];
//     for (i, pauli) in circuit.storage.iter() {
//         check[*i] = pauli.sum_up(&measurements.0);
//     }
//     let check: Live<PauliTuple> = BufferedVector::from(check).into();
//     // println!("{:?}", a);

//     assert_eq!(check, live_circuit.tracker);
// }

// {{ helpers to perform the checks
// fn check_graph(
//     graph: &PartialOrderGraph,
//     storage: &Storage,
//     measurements: &[usize],
// ) -> Result<(), String> {
//     fn check(
//         dep: (usize, bool),
//         measured: &HashMap<usize, ()>,
//         measurements: &[usize],
//     ) -> Result<(), String> {
//         if !dep.1
//             || measured
//                 .contains_key(measurements.get(dep.0).expect("missing measurement"))
//         {
//             Ok(())
//         } else {
//             Err(format!("{dep:?}"))
//         }
//     }

//     fn node_check(
//         node: &usize,
//         deps: &Vec<usize>,
//         storage: &Storage,
//         measurements: &[usize],
//         measured: &HashMap<usize, ()>,
//     ) -> Result<(), String> {
//         for dep in deps {
//             if !measured.contains_key(dep) {
//                 return Err("{dep:?}".to_string());
//             }
//         }
//         let pauli = storage.get(node).expect("node does not exist");
//         // we explicitly do not use xor(left, right), because that's what we are doing
//         // in the get_ordering function; here we keep it as simple is
//         // possible
//         // println!("{:?}", pauli.z);
//         for dep in pauli.z.iter_vals().enumerate() {
//             check(dep, measured, measurements).map_err(|e| format!("left: {e}"))?
//         }
//         for dep in pauli.x.iter_vals().enumerate() {
//             check(dep, measured, measurements).map_err(|e| format!("right: {e}"))?
//         }
//         Ok(())
//     }

//     let mut measured = HashMap::<usize, ()>::new();
//     let mut iter = graph.iter().peekable();

//     while let Some(this_layer) = iter.next() {
//         if let Some(next_layer) = iter.peek() {
//             for (node, deps) in *next_layer {
//                 // if a node in the next_layer could be measured, we fail because then
//                 // it should be in this_layer, since we want to be optimal
//                 if node_check(node, deps, storage, measurements, &measured).is_ok() {
//                     return Err(format!(
//                         "not optimal: {node:?}, {deps:?}, \
//                          {measured:?}\n{graph:#?}\n{storage:#?}"
//                     ));
//                 }
//             }
//         }
//         for (node, deps) in this_layer {
//             // if a node in this_layer can't be measured, we fail because then the
//             // dependency graph would be wrong
//             match node_check(node, deps, storage, measurements, &measured) {
//                 Ok(_) => (),
//                 Err(e) => {
//                     return Err(format!(
//                         "not sufficient: {e}\n{node:?}, {deps:?}, \
//                          {measured:?}\n{graph:?}\n{storage:#?}"
//                     ));
//                 },
//             }
//             measured.insert(*node, ());
//         }
//     }
//     Ok(())
// }
// }}

// a instructor that defines the a tracking circuit based on some operations generated
// with proptest
pub struct Instructor {
    used: usize,
    memory: Vec<usize>,
    operations: Vec<Operation>,
}

impl Instructor {
    pub fn new(init: usize, operations: Vec<Operation>) -> Self {
        Self {
            used: init,
            memory: (0..init).collect(),
            operations,
        }
    }

    // fn reinit(&mut self, init: usize) {
    //     self.used = init;
    //     self.memory = (0..init).collect();
    // }

    pub fn apply<C, T, S>(
        &mut self,
        circuit: &mut TrackedCircuit<C, T, S>,
        measurements: &mut impl Measurements<TrackedCircuit<C, T, S>>,
    ) where
        C: CliffordCircuit,
        T: Tracker,
        TrackedCircuit<C, T, S>: ExtendCircuit<Output = C::Outcome>,
    {
        for op in self.operations.iter() {
            // for small circuits, we loose some ops
            if self.memory.is_empty() {
                Self::new_qubit(&mut self.memory, &mut self.used, circuit)
            }
            // println!("{:?}", op);

            match *op {
                Operation::I(b) => circuit.id(self.mem_idx(b)),
                Operation::X(b) => circuit.x(self.mem_idx(b)),
                Operation::Y(b) => circuit.y(self.mem_idx(b)),
                Operation::Z(b) => circuit.z(self.mem_idx(b)),
                Operation::S(b) => circuit.s(self.mem_idx(b)),
                Operation::Sdg(b) => circuit.sdg(self.mem_idx(b)),
                Operation::Sz(b) => circuit.sz(self.mem_idx(b)),
                Operation::Szdg(b) => circuit.szdg(self.mem_idx(b)),
                Operation::Hxy(b) => circuit.hxy(self.mem_idx(b)),
                Operation::H(b) => circuit.h(self.mem_idx(b)),
                Operation::Sy(b) => circuit.sy(self.mem_idx(b)),
                Operation::Sydg(b) => circuit.sydg(self.mem_idx(b)),
                Operation::SH(b) => circuit.sh(self.mem_idx(b)),
                Operation::HS(b) => circuit.hs(self.mem_idx(b)),
                Operation::SHS(b) => circuit.shs(self.mem_idx(b)),
                Operation::Sx(b) => circuit.sx(self.mem_idx(b)),
                Operation::Sxdg(b) => circuit.sxdg(self.mem_idx(b)),
                Operation::Hyz(b) => circuit.hyz(self.mem_idx(b)),
                Operation::Cz(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.cz(a, b),
                    None => continue,
                },
                Operation::Cx(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.cx(a, b),
                    None => continue,
                },
                Operation::Cy(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.cy(a, b),
                    None => continue,
                },
                Operation::Swap(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.swap(a, b),
                    None => continue,
                },
                Operation::ISwap(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.iswap(a, b),
                    None => continue,
                },
                Operation::ISwapdg(a, b) => match self.double_mem_idx(a, b) {
                    Some((a, b)) => circuit.iswapdg(a, b),
                    None => continue,
                },
                Operation::Rz(b) => {
                    let bit = self.mem_idx(b);
                    measurements
                        .store(bit, circuit.z_rotation_teleportation(bit, self.used));
                    let pos_in_mem = self.idx(b);
                    self.memory[pos_in_mem] = self.used;
                    self.used += 1;
                },
                Operation::TeleportedX(a, b) => {
                    if let Some(pos_in_mem) = self.pauli_teleportation(
                        a,
                        b,
                        PauliTuple::X,
                        circuit,
                        measurements,
                    ) {
                        self.memory.swap_remove(pos_in_mem % self.memory.len());
                    }
                },
                Operation::TeleportedY(a, b) => {
                    if let Some(pos_in_mem) = self.pauli_teleportation(
                        a,
                        b,
                        PauliTuple::Y,
                        circuit,
                        measurements,
                    ) {
                        self.memory.swap_remove(pos_in_mem % self.memory.len());
                    }
                },
                Operation::TeleportedZ(a, b) => {
                    if let Some(pos_in_mem) = self.pauli_teleportation(
                        a,
                        b,
                        PauliTuple::Z,
                        circuit,
                        measurements,
                    ) {
                        self.memory.swap_remove(pos_in_mem % self.memory.len());
                    }
                },
                Operation::Measure(b) => {
                    circuit.measure(self.mem_idx(b));
                    self.memory.swap_remove(b % self.memory.len());
                },
                Operation::NewQubit(_) => {
                    Self::new_qubit(&mut self.memory, &mut self.used, circuit)
                },
            }
        }
    }

    fn idx(&self, bit: usize) -> usize {
        bit % self.memory.len()
    }
    fn mem_idx(&self, bit: usize) -> usize {
        self.memory[self.idx(bit)]
    }
    fn double_mem_idx(&self, bit_a: usize, bit_b: usize) -> Option<(usize, usize)> {
        let len = self.memory.len();
        // for small circuits, we loose some ops
        if len == 1 {
            return None;
        }
        let a = bit_a % len;
        let mut b = bit_b % len;
        if a == b {
            // this destroys some randomness; should barely happen for large circuits
            b = (bit_b + 1) % len;
        }
        Some((self.memory[a], self.memory[b]))
    }

    fn new_qubit<C, T, S>(
        memory: &mut Vec<usize>,
        used: &mut usize,
        circuit: &mut TrackedCircuit<C, T, S>,
    ) where
        C: CliffordCircuit,
        T: Tracker,
        TrackedCircuit<C, T, S>: ExtendCircuit,
    {
        circuit.new_qubit(*used);
        memory.push(*used);
        *used += 1;
    }

    fn pauli_teleportation<C, T, S>(
        &self,
        bit_a: usize,
        bit_b: usize,
        pauli: PauliTuple,
        circuit: &mut TrackedCircuit<C, T, S>,
        measurements: &mut impl Measurements<TrackedCircuit<C, T, S>>,
    ) -> Option<usize>
    where
        TrackedCircuit<C, T, S>: ExtendCircuit,
    {
        if let Some((a, b)) = self.double_mem_idx(bit_a, bit_b) {
            measurements.store(a, circuit.pauli_teleportation(a, b, pauli));
            Some(self.idx(bit_a))
        } else {
            None
        }
    }
}

pub trait Measurements<T: ExtendCircuit> {
    fn store(&mut self, bit: usize, result: T::Output);
}
pub struct WhereMeasured(pub Vec<usize>);
impl Measurements<TrackedCircuit<DummyCircuit, Frames<Storage>, Storage>>
    for WhereMeasured
{
    fn store(&mut self, bit: usize, _: ()) {
        self.0.push(bit)
    }
}
struct ResultMeasured(Vec<bool>);
impl Measurements<TrackedCircuit<RandomMeasurementCircuit, Live<PauliTuple>, ()>>
    for ResultMeasured
{
    fn store(&mut self, _: usize, result: bool) {
        self.0.push(result);
    }
}

pub trait ExtendCircuit {
    type Output;
    fn z_rotation_teleportation(&mut self, origin: usize, new: usize) -> Self::Output;
    fn new_qubit(&mut self, bit: usize);
    fn pauli_teleportation(
        &mut self,
        origin: usize,
        new: usize,
        pauli: PauliTuple,
    ) -> Self::Output;
}
impl ExtendCircuit for TrackedCircuit<DummyCircuit, Frames<Storage>, Storage> {
    type Output = ();
    fn z_rotation_teleportation(&mut self, origin: usize, new: usize) {
        self.tracker.new_qubit(new);
        self.cx(origin, new);
        self.move_z_to_z(origin, new);
        self.measure_and_store(origin).1.unwrap();
        self.track_z(new);
    }
    fn new_qubit(&mut self, bit: usize) {
        self.tracker.new_qubit(bit);
    }
    fn pauli_teleportation(
        &mut self,
        origin: usize,
        new: usize,
        pauli: PauliTuple,
    ) -> Self::Output {
        self.track_pauli(new, pauli);
        self.measure_and_store(origin).1.unwrap();
    }
}
impl ExtendCircuit for TrackedCircuit<RandomMeasurementCircuit, Live<PauliTuple>, ()> {
    type Output = bool;
    fn z_rotation_teleportation(&mut self, origin: usize, new: usize) -> bool {
        self.tracker.new_qubit(new);
        self.cx(origin, new);
        self.move_z_to_z(origin, new);
        let res = self.circuit.measure(origin);
        if res {
            self.track_z(new);
        };
        res
    }
    fn new_qubit(&mut self, bit: usize) {
        self.tracker.new_qubit(bit);
    }
    fn pauli_teleportation(
        &mut self,
        origin: usize,
        new: usize,
        pauli: PauliTuple,
    ) -> Self::Output {
        let res = self.circuit.measure(origin);
        if res {
            self.track_pauli(new, pauli);
        }
        res
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum Operation {
    I(usize),
    X(usize),
    Y(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    Sz(usize),
    Szdg(usize),
    Hxy(usize),
    H(usize),
    Sy(usize),
    Sydg(usize),
    SH(usize),
    HS(usize),
    SHS(usize),
    Sx(usize),
    Sxdg(usize),
    Hyz(usize),
    Cz(usize, usize),
    Cx(usize, usize),
    Cy(usize, usize),
    Swap(usize, usize),
    ISwap(usize, usize),
    ISwapdg(usize, usize),
    TeleportedX(usize, usize),
    TeleportedY(usize, usize),
    TeleportedZ(usize, usize),
    Rz(usize),
    Measure(usize),
    NewQubit(usize),
}

// the creation of the operations with proptest

fn operation() -> impl Strategy<Value = Operation> {
    prop_oneof![
        1 => any::<usize>().prop_map(Operation::I),
        1 => any::<usize>().prop_map(Operation::X),
        1 => any::<usize>().prop_map(Operation::Y),
        1 => any::<usize>().prop_map(Operation::Z),
        15 => any::<usize>().prop_map(Operation::S),
        15 => any::<usize>().prop_map(Operation::Sdg),
        15 => any::<usize>().prop_map(Operation::Sz),
        15 => any::<usize>().prop_map(Operation::Szdg),
        5 => any::<usize>().prop_map(Operation::Hxy),
        50 => any::<usize>().prop_map(Operation::H),
        5 => any::<usize>().prop_map(Operation::Sy),
        5 => any::<usize>().prop_map(Operation::Sydg),
        5 => any::<usize>().prop_map(Operation::SH),
        5 => any::<usize>().prop_map(Operation::HS),
        5 => any::<usize>().prop_map(Operation::SHS),
        5 => any::<usize>().prop_map(Operation::Sx),
        5 => any::<usize>().prop_map(Operation::Sxdg),
        5 => any::<usize>().prop_map(Operation::Hyz),
        30 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::Cx(a, b)),
        7 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::Cy(a, b)),
        30 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::Cz(a, b)),
        40 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::Swap(a, b)),
        5 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::ISwap(a, b)),
        5 => (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Operation::ISwapdg(a, b)),
        15 => (any::<usize>(), any::<usize>())
                 .prop_map(|(a, b)| Operation::TeleportedX(a, b)),
        15 => (any::<usize>(), any::<usize>())
                 .prop_map(|(a, b)| Operation::TeleportedY(a, b)),
        15 => (any::<usize>(), any::<usize>())
                 .prop_map(|(a, b)| Operation::TeleportedZ(a, b)),
        100 => any::<usize>().prop_map(Operation::Rz),
        2 => any::<usize>().prop_map(Operation::Measure),
        2 => any::<usize>().prop_map(Operation::NewQubit),
    ]
}

pub fn fixed_num_vec_operation(
    mut num_operations: usize,
) -> impl Strategy<Value = Vec<Operation>> {
    let mut res = Vec::new();
    while num_operations > 0 {
        res.push(operation());
        num_operations -= 1;
    }
    res
}

// fn vec_operation(max_num_operations: usize) -> impl Strategy<Value = Vec<Operation>> {
//     (0..max_num_operations).prop_flat_map(fixed_num_vec_operation)
// }
