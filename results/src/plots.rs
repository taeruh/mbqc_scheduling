use std::time::{Duration, Instant};

use bitvec::vec::BitVec;
use mbqc_scheduling::{interface, probabilistic::AcceptFunc, search::SpacialGraph};
use pauli_tracker::{
    collection::{Base, Iterable, NaiveVector},
    pauli::PauliStack,
    tracker::frames::induced_order::{self, PartialOrderGraph},
};
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use serde::Serialize;

use crate::{NCPUS, NUM_AVERAGE};

pub mod density;
pub mod node;

pub enum Args {
    Node(node::Args),
    Density(density::Args),
}

fn do_it(
    size: usize,
    edge_density: impl Density,
    correction_density: impl Density,
    timeout: Duration,
    rng: &mut impl Rng,
) -> ([(f64, f64); 4], Duration) {
    let mut results = vec![Vec::with_capacity(NUM_AVERAGE); 4];
    let mut means = [0.0; 4];

    let mut averaged_time = Duration::default();
    for _ in 0..NUM_AVERAGE {
        let graph = get_graph(edge_density, size, rng);
        let order = get_order(correction_density, size, rng);

        let time_optimal =
            interface::run(graph.clone(), order.clone(), false, None, 1, None, None);
        let time = Instant::now();
        let space_optimal_approx = interface::run(
            graph,
            order,
            true,
            Some(timeout),
            NCPUS,
            None,
            Some((AcceptFunc::BuiltinHeavyside, Some(rng.gen()))),
        );
        averaged_time += time.elapsed();

        // if the accept function was to aggressive we may not have a path at all
        if let Some(time_optimal) = time_optimal.first() {
            results[0].push(time_optimal.time);
            results[1].push(time_optimal.space);
        }
        if let Some(space_optimal_approx) = space_optimal_approx.last() {
            results[2].push(space_optimal_approx.time);
            results[3].push(space_optimal_approx.space);
        }

        for (result, mean) in results.iter_mut().zip(means.iter_mut()) {
            *mean += *result.last().unwrap() as f64;
        }
    }

    let mut ret = [Default::default(); 4];
    for (i, (result, mut mean)) in results.into_iter().zip(means.into_iter()).enumerate()
    {
        let actual_num_average = result.len();
        if actual_num_average as f64 / (NUM_AVERAGE as f64) < 0.9 {
            println!(
                "Warning: less 90% results for size {}; only {} results instead of {}",
                size, actual_num_average, NUM_AVERAGE
            );
        }
        mean /= actual_num_average as f64;
        let deviatian = (result.iter().map(|e| (*e as f64 - mean).powi(2)).sum::<f64>()
            / actual_num_average as f64)
            .sqrt();
        ret[i] = (mean, deviatian)
    }

    (
        ret,
        Duration::from_nanos(
            (averaged_time.as_nanos() / NUM_AVERAGE as u128).try_into().unwrap(),
        ),
    )
}

trait Density: Copy {
    fn get(&self, size: usize) -> f64;
}

macro_rules! density_types {
    ($(($density_type:ident,$name:literal),)*) => {
        $(
            #[derive(Copy, Clone, Debug, Serialize)]
            pub struct $density_type {
                #[serde(rename = $name)]
                factor: f64,
            }
            impl $density_type {
                pub fn new(factor: f64) -> Self {
                    assert!(
                        (0.0..=1.0).contains(&factor),
                        "density must be between 0 and 1, but it is: {factor}"
                    );
                    Self { factor }
                }
            }
        )*
    };
}
density_types!(
    (ConstantDensity, "constant"),
    (ReziprocalLinearDensity, "reziprocal_linear"),
);

impl Density for ConstantDensity {
    fn get(&self, _: usize) -> f64 {
        self.factor
    }
}
impl Density for ReziprocalLinearDensity {
    fn get(&self, size: usize) -> f64 {
        self.factor / (size as f64)
    }
}

fn get_graph(density: impl Density, size: usize, rng: &mut impl Rng) -> SpacialGraph {
    if size == 0 {
        return vec![];
    }

    let density = density.get(size);
    let unform = Uniform::new(0.0, 1.0);

    let mut ret = vec![vec![]; size];
    for this in 0..size - 1 {
        for other in this + 1..size {
            if unform.sample(rng) < density {
                ret[this].push(other);
                ret[other].push(this);
            }
        }
    }
    ret
}

fn get_order(
    density: impl Density,
    size: usize,
    rng: &mut impl Rng,
) -> PartialOrderGraph {
    if size == 0 {
        return vec![];
    }

    let density = density.get(size);
    let unform = Uniform::new(0.0, 1.0);

    let mut frames = NaiveVector::from(vec![PauliStack::<BitVec>::zeros(size); size]);
    let mut pool = (0..size).collect::<Vec<_>>();
    let mut map = Vec::with_capacity(size);

    while !pool.is_empty() {
        let index = rng.gen_range(0..pool.len());
        let bit = pool.swap_remove(index);
        // the rounding errors of the following are too large for small sizes
        // pool.choose_multiple(rng, (density * (pool.len() as f64)).round() as usize);
        let corrections = pool
            .iter()
            .filter(|_| unform.sample(rng) < density)
            .collect::<Vec<_>>();
        for &correction in corrections {
            frames.get_mut(correction).unwrap().z.set(map.len(), true);
        }
        map.push(bit);
    }
    induced_order::get_order(frames.iter_pairs(), map.as_ref())
}
