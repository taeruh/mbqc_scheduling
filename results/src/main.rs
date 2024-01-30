use std::{
    env,
    fs::{self, File},
    time::Duration,
};

use bitvec::vec::BitVec;
use mbqc_scheduling::{
    interface::{self, SpacialGraph},
    probabilistic::AcceptFunc,
};
use pauli_tracker::{
    collection::{Base, Iterable, NaiveVector},
    pauli::PauliStack,
    tracker::frames::induced_order::{self, PartialOrderGraph},
};
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_pcg::Pcg64;
use serde::Serialize;

const MAX_SIZE: usize = 20;
const NUM_AVERAGE: usize = 1000;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    assert_eq!(args.len(), 3, "Usage: <edge_density> <correction_density>");
    let edge_density = Density::new(args[1].parse::<f64>().unwrap());
    let correction_density = Density::new(args[2].parse::<f64>().unwrap());

    let output_file = format!("output/{}_{}.json", edge_density.0, correction_density.0);

    // if the number of threads passed to `run` below is 1, then one could replace this
    // seed with the seed in the output_file to reproduce the result, however, the final
    // results are not produced single threaded, so this is pointless (cf. doc of the run
    // function)
    let seed = Pcg64::from_entropy().gen();

    let mut rng = Pcg64::seed_from_u64(seed);
    let mut averaged_results = Vec::with_capacity(MAX_SIZE);

    for size in 1..MAX_SIZE + 1 {
        let mut results = vec![Vec::with_capacity(MAX_SIZE); 4];
        let mut means = [0.0; 4];

        for _ in 0..NUM_AVERAGE {
            let graph = get_graph(edge_density, size, &mut rng);
            let order = get_order(correction_density, size, &mut rng);
            let time_optimal =
                interface::run(graph.clone(), order.clone(), false, None, 1, None, None);
            let space_optimal_approx = interface::run(
                graph,
                order,
                true,
                // Some(Duration::from_millis(2000)),
                None,
                10,
                None,
                Some((AcceptFunc::BuiltinHeavyside, Some(rng.gen()))),
            );

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

        averaged_results.push((
            size,
            results
                .into_iter()
                .zip(means.into_iter())
                .map(|(result, mut mean)| {
                    let actual_num_average = result.len();
                    if actual_num_average as f64 / (NUM_AVERAGE as f64) < 0.9 {
                        println!(
                            "Warning: less 90% results for size {}; only {} results \
                             instead of {}",
                            size, actual_num_average, NUM_AVERAGE
                        );
                    }
                    mean /= actual_num_average as f64;
                    let deviatian =
                        (result.iter().map(|e| (*e as f64 - mean).powi(2)).sum::<f64>()
                            / actual_num_average as f64)
                            .sqrt();
                    (mean, deviatian)
                })
                .collect(),
        ));
    }

    let output = Output {
        edge_density,
        correction_density,
        seed,
        results: averaged_results,
    };

    fs::create_dir_all("output").unwrap();
    serde_json::to_writer(File::create(output_file).unwrap(), &output).unwrap();
}

#[derive(Serialize)]
struct Output {
    edge_density: Density,
    correction_density: Density,
    seed: u64,
    results: Vec<(usize, Vec<(f64, f64)>)>,
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(transparent)]
struct Density(f64);

impl Density {
    fn new(density: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&density),
            "density must be between 0 and 1, but it is: {density}"
        );
        Density(density)
    }
}

fn get_graph(density: Density, size: usize, rng: &mut impl Rng) -> SpacialGraph {
    if size == 0 {
        return vec![];
    }

    let density = density.0;
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

fn get_order(density: Density, size: usize, rng: &mut impl Rng) -> PartialOrderGraph {
    if size == 0 {
        return vec![];
    }

    let density = density.0;

    let mut frames = NaiveVector::from(vec![PauliStack::<BitVec>::zeros(size); size]);
    let mut pool = (0..size).collect::<Vec<_>>();
    let mut map = Vec::with_capacity(size);

    while !pool.is_empty() {
        let index = rng.gen_range(0..pool.len());
        let bit = pool.swap_remove(index);
        let corrections =
            pool.choose_multiple(rng, (density * (pool.len() as f64)).round() as usize);
        for &correction in corrections {
            frames.get_mut(correction).unwrap().z.set(map.len(), true);
        }
        map.push(bit);
    }
    induced_order::get_order(frames.iter_pairs(), map.as_ref())
}
