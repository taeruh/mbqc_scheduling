/*!
Search for the optimal initialization-measurement paths. This module has nothing to do
with pauli tracking anymore, except that we take the [DependencyGraph] as input.
*/

// the logic of the path search is basically described in the documentation of the
// schedule module; here I'm basically just multithreading it

use std::{
    cmp,
    collections::HashMap,
    sync::{
        mpsc,
        Arc,
        Mutex,
        OnceLock,
    },
    time::Instant,
};

use anyhow::Result;
use pauli_tracker::tracker::frames::dependency_graph::DependencyGraph;
use rand::{
    distributions::{
        Distribution,
        Uniform,
    },
    SeedableRng,
};
use rand_pcg::Pcg64;
use scoped_threadpool::Pool;

use crate::{
    interface::Paths,
    probabilistic::AcceptFn,
    scheduler::{
        space::{
            Graph,
            GraphBuffer,
        },
        time::{
            DependencyBuffer,
            Partitioner,
            PathGenerator,
        },
        tree::{
            Focus,
            FocusIterator,
            Step,
            Sweep,
        },
        Scheduler,
    },
};

pub fn get_time_optimal(
    deps: DependencyGraph,
    mut dependency_buffer: DependencyBuffer,
    graph_buffer: GraphBuffer,
) -> Result<Paths> {
    let mut scheduler = Scheduler::<Vec<usize>>::new(
        PathGenerator::from_dependency_graph(deps, &mut dependency_buffer, None),
        Graph::new(&graph_buffer),
    );

    let mut path = Vec::new();
    let mut max_memory = 0;

    while !scheduler.time().measurable().is_empty() {
        let measurable_set = scheduler.time().measurable().clone();
        scheduler.focus_inplace(&measurable_set)?;
        path.push(measurable_set);
        max_memory = cmp::max(max_memory, scheduler.space().max_memory());
    }

    Ok(vec![(path.len(), (max_memory, path))])
}

type OnePath = Vec<Vec<usize>>;
type MappedPaths = HashMap<usize, (usize, Vec<Vec<usize>>)>;

static ACCEPT: OnceLock<AcceptFn> = OnceLock::new();

#[allow(clippy::too_many_arguments)]
pub fn search(
    deps: DependencyGraph,
    mut dependency_buffer: DependencyBuffer,
    graph_buffer: GraphBuffer,
    nthreads: u16,
    probabilistic: Option<AcceptFn>,
    num_bits: usize,
    task_bound: i64,
    debug: bool,
) -> Result<Paths> {
    let scheduler = Scheduler::<Partitioner>::new(
        PathGenerator::from_dependency_graph(deps, &mut dependency_buffer, None),
        Graph::new(&graph_buffer),
    );

    let probabilistic = if let Some(probabilistic) = probabilistic {
        ACCEPT.get_or_init(|| probabilistic);
        true
    } else {
        false
    };

    let results = if nthreads < 3 {
        let (result, _) = if probabilistic {
            probabilistic_search_single_task(scheduler, num_bits, None, None)
        } else {
            search_single_task(scheduler, num_bits, None, None)
        };
        result
    } else {
        threaded_search(nthreads, num_bits, scheduler, task_bound, debug, probabilistic)
    }?;

    let mut filtered_results = HashMap::new();
    let mut best_memory = vec![num_bits + 1; num_bits + 1];
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

    let mut sorted = filtered_results.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|(len, _)| *len);

    // println!("sorted:");
    // for s in sorted.iter() {
    //     println!("{:?}", s);
    // }
    // println!("results:");
    // for r in results.iter() {
    //     println!("{:?}", r);
    // }

    Ok(sorted)
}

// cf. pauli_tracker::scheduler doc examples
fn search_single_task(
    scheduler: Scheduler<Partitioner>,
    num_bits: usize,
    // following two only needed for parallel search
    init_path: Option<OnePath>,
    predicates: Option<Vec<usize>>,
) -> (Result<MappedPaths>, Vec<usize>) {
    let mut results = HashMap::new();
    let mut current_path = init_path.unwrap_or_default();
    let mut best_memory = predicates.unwrap_or_else(|| vec![num_bits + 1; num_bits + 1]);
    let mut scheduler = scheduler.into_iter();
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
    }

    (Ok(results), best_memory)
}

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

fn probabilistic_search_single_task(
    scheduler: Scheduler<Partitioner>,
    num_bits: usize,
    // following two only needed for parallel search
    init_path: Option<OnePath>,
    predicates: Option<Vec<usize>>,
) -> (Result<MappedPaths>, Vec<usize>) {
    let mut results = HashMap::new();
    let mut current_path = init_path.unwrap_or_default();
    let mut best_memory = predicates.unwrap_or_else(|| vec![num_bits + 1; num_bits + 1]);
    let mut scheduler = scheduler.into_iter();

    let mut rng = Pcg64::from_entropy();
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
    }

    (Ok(results), best_memory)
}

#[allow(clippy::too_many_arguments)]
fn probabilistic_forward(
    measure: Vec<usize>,
    scheduler: &mut Sweep<Scheduler<Partitioner>>,
    best_memory: &[usize],
    current_path: &mut OnePath,
    last_cur_mem: usize,
    last_max_mem: usize,
    rng: &mut impl rand::Rng,
    dist: &Uniform<f64>,
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
        // TODO: use unwrap_unchecked; should be safe
        let accept = ACCEPT.get().unwrap()(
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

#[allow(clippy::too_many_arguments)]
fn task(
    scheduler: Scheduler<Partitioner>,
    best_memory: Vec<usize>,
    ntasks: i64,
    sender: mpsc::Sender<(Vec<usize>, MappedPaths)>,
    measure: Option<Vec<usize>>,
    num_bits: usize,
    debug: bool,
    probabilistic: bool,
) -> Result<()> {
    let start = if debug {
        println!("start {ntasks:?}: measure {measure:?}; best_memory {best_memory:?}");
        Some(Instant::now())
    } else {
        None
    };

    let (results, new_best_memory) = if probabilistic {
        probabilistic_search_single_task(
            scheduler,
            num_bits,
            measure.map(|e| vec![e]),
            Some(best_memory),
        )
    } else {
        search_single_task(
            scheduler,
            num_bits,
            measure.map(|e| vec![e]),
            Some(best_memory),
        )
    };

    if let Some(start) = start {
        println!(
            "done {ntasks:?}: time {:?}; results {:?}",
            Instant::now() - start,
            results.as_ref().unwrap()
        );
    }
    sender.send((new_best_memory, results?)).expect("send failure");
    Ok(())
}

fn threaded_search(
    nthreads: u16,
    num_bits: usize,
    mut scheduler: Scheduler<Partitioner>,
    task_bound: i64,
    debug: bool,
    probabilistic: bool,
) -> Result<MappedPaths> {
    // there will be one thread which only collects the results and updates the shared
    // best_memory array, the other threads do the actual search tasks

    let mut pool = Pool::new(nthreads as u32);
    let (sender, receiver) = mpsc::channel::<(Vec<usize>, MappedPaths)>();

    let best_memory = Arc::new(Mutex::new(vec![num_bits + 1; num_bits + 1]));
    let results: Arc<Mutex<MappedPaths>> = Arc::new(Mutex::new(HashMap::new()));

    let mut ntasks = 0;
    pool.scoped(|scope| {
        // update best_memory and the results
        let clone_best_memory = best_memory.clone();
        let clone_results = results.clone();
        scope.execute(move || {
            while let Ok((new_best_memory, mut new_results)) = receiver.recv() {
                let mut best_memory =
                    clone_best_memory.lock().expect("failed to lock best_memory");
                let mut results = clone_results.lock().expect("failed to lock results");

                for (i, (o, n)) in best_memory.iter_mut().zip(new_best_memory).enumerate()
                {
                    if *o > n {
                        *o = n;
                        // we cannot just unwrap, because when we do the
                        // Step::Backward, we also update the best_memory for all
                        // paths with a longer path length, but we might not have
                        // collected a result for them (which is okay there)
                        if let Some(e) = new_results.remove(&i) {
                            results.insert(i, e);
                        }
                    }
                }
            }
        });

        // NOTE: if probabilistic is true, one should maybe potentially skip here; but
        // maybe it is actually good to not do it, because this increases the probability
        // that we get at least some results
        while let Some((scheduler_focused, init_measure)) = scheduler.next_and_focus() {
            // println!("{:?}", ntasks);
            let sender = sender.clone();
            let clone_best_memory = best_memory.clone();
            // search tasks
            scope.execute(move || {
                // don't do that in the search fn call, because this would create a
                // temporary varialbe of the MutexGuard, I think, which is only
                // dropped when the function returns -> one task would block all
                // others tasks
                let best_memory = clone_best_memory
                    .lock()
                    .expect("failed to lock best_memory for task")
                    .to_vec();
                task(
                    scheduler_focused,
                    best_memory,
                    ntasks,
                    sender,
                    Some(init_measure),
                    num_bits,
                    debug,
                    probabilistic,
                )
                .expect("task failed");
            });
            ntasks += 1;
            if ntasks == task_bound {
                break;
            }
        }

        // remaining search tasks; note that this one takes ownership of
        // sender (i.e., it is droped and we are not endlessly waiting when trying
        // to receive from the channel)
        scope.execute(move || {
            let best_memory = best_memory
                .lock()
                .expect("failed to lock best_memory for final task")
                .to_vec();
            task(
                scheduler,
                best_memory,
                -1,
                sender,
                None,
                num_bits,
                debug,
                probabilistic,
            )
            .expect("final task failed");
        });
        // drop(sender);
    });

    Ok(Arc::into_inner(results)
        .expect("failed to move out of Arc results")
        .into_inner()
        .expect("failed to move out of Mutex results"))
}
