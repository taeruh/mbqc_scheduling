/*!
Search for the optimal initialization-measurement paths. This module has nothing to do
with pauli tracking anymore, except that we take the [DependencyGraph] as input.
*/

// the logic of the path search is basically described in the documentation of the
// schedule module; here I'm basically just multithreading it

use std::{cmp, collections::HashMap, time::Duration};

use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};
use rand_pcg::Pcg64;

use crate::{
    interface::{Path, RefPartialOrderGraph},
    probabilistic::{Accept, AcceptBox},
    scheduler::{
        space::Graph,
        time::{DependencyBuffer, Partitioner, PathGenerator},
        tree::{Focus, FocusIterator, Step, Sweep},
        Partition, Scheduler,
    },
    timer::Timer,
};

mod threaded;
pub type SpacialGraph = Vec<Vec<usize>>;
pub type RefSpacialGraph = [Vec<usize>];

pub fn get_time_optimal(
    spacial_graph: &RefSpacialGraph,
    time_ordering: &RefPartialOrderGraph,
) -> Vec<Path> {
    let mut dependency_buffer = DependencyBuffer::new(spacial_graph.len());
    // let graph_buffer = GraphBuffer::from_sparse(spacial_graph);
    let graph_buffer = spacial_graph;

    let mut scheduler = Scheduler::<Vec<usize>>::new(
        PathGenerator::from_dependency_graph(time_ordering, &mut dependency_buffer, None),
        Graph::new(graph_buffer),
    );

    let mut path = Vec::new();
    let mut max_memory = 0;

    while !scheduler.time().measurable().is_empty() {
        let measurable_set = scheduler.time().measurable().clone();
        scheduler.focus_inplace(&measurable_set).expect(
            "weird error; there must be something wrong with the dependency graph",
        );
        path.push(measurable_set);
        max_memory = cmp::max(max_memory, scheduler.space().max_memory());
    }

    vec![Path {
        time: path.len(),
        space: max_memory,
        steps: path,
    }]
}

type OnePath = Vec<Vec<usize>>;
type MappedPaths = HashMap<usize, (usize, Vec<Vec<usize>>)>;

pub fn search(
    spacial_graph: &RefSpacialGraph,
    time_ordering: &RefPartialOrderGraph,
    timeout: Option<Duration>,
    nthreads: u16,
    probabilistic: Option<(AcceptBox, Option<u64>)>,
    task_bound: i64,
) -> Vec<Path> {
    let num_bits = spacial_graph.len();
    let mut dependency_buffer = DependencyBuffer::new(num_bits);
    // let graph_buffer = GraphBuffer::from_sparse(spacial_graph);
    let graph_buffer = spacial_graph;
    let scheduler = Scheduler::<Partitioner>::new(
        PathGenerator::from_dependency_graph(time_ordering, &mut dependency_buffer, None),
        Graph::new(graph_buffer),
    );

    let mut timer = Timer::new();
    if let Some(timeout) = timeout {
        timer.start(timeout);
    }

    let results = if nthreads < 2 {
        let (result, _) = if let Some(accept_func) = probabilistic {
            do_probabilistic_search(scheduler.into_iter(), num_bits, &timer, accept_func)
        } else {
            do_search(scheduler.into_iter(), num_bits, &timer)
        };
        result
    } else {
        threaded::search(nthreads, num_bits, scheduler, task_bound, probabilistic, &timer)
    };

    let mut filtered_results = HashMap::new();
    let mut best_memory = vec![usize::MAX; num_bits + 1];
    for i in 0..best_memory.len() {
        if let Some((mem, _)) = results.get(&i) {
            let m = best_memory[i];
            if *mem < m {
                filtered_results.insert(i, results.get(&i).unwrap().clone());
                for m in best_memory[i..].iter_mut() {
                    *m = *mem;
                }
            }
        }
    }

    let mut sorted = filtered_results
        .into_iter()
        .map(|(time, (space, steps))| Path { time, space, steps })
        .collect::<Vec<_>>();
    sorted.sort_by_key(|Path { time, .. }| *time);

    sorted
}

// cf. crate::scheduler doc examples
fn do_search(
    mut scheduler: Sweep<Scheduler<Partition<Vec<usize>>>>,
    num_bits: usize,
    timer: &Timer,
) -> (MappedPaths, Vec<usize>) {
    let mut results = HashMap::new();
    let mut current_path = Vec::new();
    let mut best_memory = vec![usize::MAX; num_bits + 1];
    while let Some(step) = scheduler.next() {
        match step {
            Step::Forward(measure) => {
                if forward(measure, &mut scheduler, &best_memory, &mut current_path) {
                    break;
                }
            },
            Step::Backward(leaf) => {
                backward(leaf, &mut current_path, &mut best_memory, &mut results);
            },
        }
        if timer.finished() {
            tracing::info!("timer: timeout");
            break;
        }
    }

    (results, best_memory)
}

#[inline]
fn minimum_path_length(
    time: &PathGenerator<Partitioner>,
    current_path: &OnePath,
) -> usize {
    if time.at_leaf().is_some() {
        current_path.len() + 1
    } else if time.has_unmeasureable() {
        current_path.len() + 3
    } else {
        current_path.len() + 2
    }
}

#[inline]
fn forward(
    measure: Vec<usize>,
    scheduler: &mut Sweep<Scheduler<Partitioner>>,
    best_memory: &[usize],
    current_path: &mut OnePath,
) -> bool {
    let current = scheduler.current();
    let space = current.space();
    if space.max_memory()
        >= best_memory[minimum_path_length(current.time(), current_path)]
    {
        if scheduler.skip_current().is_err() {
            return true;
        }
    } else {
        current_path.push(measure);
    }
    false
}

#[inline]
fn backward(
    leaf: Option<usize>,
    current_path: &mut OnePath,
    best_memory: &mut [usize],
    results: &mut MappedPaths,
) {
    if let Some(mem) = leaf {
        best_memory[current_path.len()] = mem;
        for m in best_memory[current_path.len() + 1..].iter_mut() {
            *m = cmp::min(*m, mem);
        }
        results.insert(current_path.len(), (mem, current_path.clone()));
    }
    current_path.pop();
}

fn do_probabilistic_search(
    mut scheduler: Sweep<Scheduler<Partition<Vec<usize>>>>,
    num_bits: usize,
    timer: &Timer,
    (accept_func, seed): (AcceptBox, Option<u64>),
) -> (MappedPaths, Vec<usize>) {
    let mut results = HashMap::new();
    let mut current_path = Vec::new();
    let mut best_memory = vec![usize::MAX; num_bits + 1];

    let mut rng = if let Some(seed) = seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };
    let dist = Uniform::new(0., 1.);

    loop {
        let space = scheduler.current().space();
        let last_cur_mem = space.current_memory();
        let last_max_mem = space.max_memory();
        if let Some(step) = scheduler.next() {
            match step {
                Step::Forward(measure) => {
                    if probabilistic_forward(
                        measure,
                        &mut scheduler,
                        &best_memory,
                        &mut current_path,
                        last_cur_mem,
                        last_max_mem,
                        &mut rng,
                        &dist,
                        &accept_func,
                    ) {
                        break;
                    }
                },
                Step::Backward(leaf) => {
                    backward(leaf, &mut current_path, &mut best_memory, &mut results);
                },
            }
        } else {
            break;
        }
        if timer.finished() {
            tracing::info!("timer: timeout");
            break;
        }
    }

    (results, best_memory)
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn probabilistic_forward(
    measure: Vec<usize>,
    scheduler: &mut Sweep<Scheduler<Partitioner>>,
    best_memory: &[usize],
    current_path: &mut OnePath,
    last_cur_mem: usize,
    last_max_mem: usize,
    rng: &mut impl rand::Rng,
    dist: &Uniform<f64>,
    accept_func: &Accept,
) -> bool {
    let current = scheduler.current();
    let space = current.space();
    let bound_best_mem = best_memory[minimum_path_length(current.time(), current_path)];
    if space.max_memory() >= bound_best_mem {
        if scheduler.skip_current().is_err() {
            return true;
        }
    } else {
        // PERF: use unwrap_unchecked; should be safe
        let accept = accept_func(
            bound_best_mem as f64,
            *best_memory.last().unwrap() as f64,
            last_max_mem as f64,
            last_cur_mem as f64,
            space.current_memory() as f64,
            current.time().num_remaining_nodes() as f64,
            space.nodes().len() as f64,
        );
        if accept >= 1. || dist.sample(rng) < accept {
            current_path.push(measure);
        } else if scheduler.skip_current().is_err() {
            return true;
        }
    }
    false
}
