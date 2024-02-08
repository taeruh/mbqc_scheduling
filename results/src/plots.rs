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
    get_full: bool,
) -> (Vec<(f64, f64)>, Duration, Duration, Option<Duration>) {
    let num_res = if get_full { 8 } else { 4 };
    let mut results = vec![Vec::with_capacity(NUM_AVERAGE); num_res];
    let mut means = vec![0.0; num_res];

    let mut time_optimal_time = Duration::default();
    let mut full_time = Duration::default();
    let mut space_optimal_approximated_time = Duration::default();
    for _ in 0..NUM_AVERAGE {
        let graph = get_graph(edge_density, size, rng);
        let order = get_order(correction_density, size, rng);

        let mut time = Instant::now();
        let time_optimal = interface::run(&graph, &order, false, None, 1, None, None);
        time_optimal_time += time.elapsed();
        time = Instant::now();
        let space_optimal_approx = interface::run(
            &graph,
            &order,
            true,
            Some(timeout),
            NCPUS,
            None,
            Some((AcceptFunc::BuiltinHeavyside, Some(rng.gen()))),
        );
        space_optimal_approximated_time += time.elapsed();
        let full = if get_full {
            time = Instant::now();
            let full = interface::run(&graph, &order, true, None, NCPUS, None, None);
            full_time += time.elapsed();
            Some(full)
        } else {
            None
        };

        let time_optimal = time_optimal.first().unwrap();
        results[0].push(time_optimal.time);
        results[1].push(time_optimal.space);
        // if the accept function was to aggressive we may not have a path at all
        if let Some(space_optimal_approx) = space_optimal_approx.last() {
            results[2].push(space_optimal_approx.time);
            results[3].push(space_optimal_approx.space);
        }
        if let Some(full) = full {
            if let Some(time_optimal) = full.first() {
                results[4].push(time_optimal.time);
                results[5].push(time_optimal.space);
            }
            if let Some(space_optimal) = full.last() {
                results[6].push(space_optimal.time);
                results[7].push(space_optimal.space);
            }
        }

        for (result, mean) in results.iter_mut().zip(means.iter_mut()) {
            *mean += *result.last().unwrap() as f64;
        }
    }

    let mut ret = vec![Default::default(); num_res];
    for (i, (result, mut mean)) in results.into_iter().zip(means.into_iter()).enumerate()
    {
        let actual_num_average = result.len();
        if actual_num_average as f64 / (NUM_AVERAGE as f64) < 0.9 {
            println!(
                "Warning: less 90% results for size {}, search {i}; only {} results \
                 instead of {}",
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
            (time_optimal_time.as_nanos() / NUM_AVERAGE as u128)
                .try_into()
                .unwrap(),
        ),
        Duration::from_nanos(
            (space_optimal_approximated_time.as_nanos() / NUM_AVERAGE as u128)
                .try_into()
                .unwrap(),
        ),
        if get_full {
            Some(Duration::from_nanos(
                (full_time.as_nanos() / NUM_AVERAGE as u128).try_into().unwrap(),
            ))
        } else {
            None
        },
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
    (ReziprocalSquareRootDensity, "reziprocal_square_root"),
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
impl Density for ReziprocalSquareRootDensity {
    fn get(&self, size: usize) -> f64 {
        self.factor / (size as f64).sqrt()
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(untagged)]
enum DensityEnum {
    Constant(ConstantDensity),
    ReziprocalLinear(ReziprocalLinearDensity),
    ReziprocalSquareRoot(ReziprocalSquareRootDensity),
}

impl DensityEnum {
    fn new(density_type: &str, factor: f64) -> Self {
        match density_type {
            "constant" => Self::Constant(ConstantDensity::new(factor)),
            "reziprocal_linear" => {
                Self::ReziprocalLinear(ReziprocalLinearDensity::new(factor))
            },
            "reziprocal_square_root" => {
                Self::ReziprocalSquareRoot(ReziprocalSquareRootDensity::new(factor))
            },
            _ => unreachable!(),
        }
    }
}

impl Density for DensityEnum {
    fn get(&self, size: usize) -> f64 {
        match self {
            Self::Constant(density) => density.get(size),
            Self::ReziprocalLinear(density) => density.get(size),
            Self::ReziprocalSquareRoot(density) => density.get(size),
        }
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
