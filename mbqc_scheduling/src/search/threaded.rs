use std::{
    cmp::Ordering,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

use rand::{distributions::Uniform, Rng, SeedableRng};
use rand_pcg::Pcg64;
use scoped_threadpool::Pool;

use super::{MappedPaths, OnePath};
use crate::{
    probabilistic::{Accept, AcceptBox},
    scheduler::{
        time::Partitioner,
        tree::{FocusIterator, Step, Sweep},
        Partition, Scheduler,
    },
    timer::Timer,
};

pub fn search(
    nthreads: u16,
    num_bits: usize,
    mut scheduler: Scheduler<Partitioner>,
    task_bound: i64,
    probabilistic: Option<(AcceptBox, Option<u64>)>,
    timer: &Timer,
) -> MappedPaths {
    let mut pool = Pool::new(nthreads as u32);

    let best_memory = Arc::new(Mutex::new(vec![usize::MAX; num_bits + 1]));
    let results: Arc<Mutex<MappedPaths>> = Arc::new(Mutex::new(HashMap::new()));
    let mut probabilistic = match probabilistic {
        Some((ref func, seed)) => Some((
            func.deref(),
            if let Some(seed) = seed {
                Pcg64::seed_from_u64(seed)
            } else {
                Pcg64::from_entropy()
            },
        )),
        None => None,
    };

    let mut ntasks = 0;
    pool.scoped(|scope| {
        // if probabilistic is true, one should maybe already skip here according to
        // the accept_func; but maybe it is actually good to not do it, because this
        // increases the probability that we get at least some results
        while let Some((scheduler_focused, init_measure)) = scheduler.next_and_focus() {
            let best_memory = best_memory.clone();
            let results = results.clone();
            let probabilistic = match probabilistic {
                Some((func, ref mut rng)) => Some((func, rng.gen())),
                None => None,
            };
            scope.execute(move || {
                task(
                    best_memory,
                    results,
                    scheduler_focused,
                    ntasks,
                    Some(init_measure),
                    timer,
                    probabilistic,
                )
            });
            ntasks += 1;
            if ntasks == task_bound {
                break;
            }
        }

        // remaining search tasks; note that this one takes ownership of best_memory
        let results = results.clone();
        let probabilistic = match probabilistic {
            Some((func, ref mut rng)) => Some((func, rng.gen())),
            None => None,
        };
        scope.execute(move || {
            task(best_memory, results, scheduler, -1, None, timer, probabilistic)
        });
    });

    Arc::into_inner(results).unwrap().into_inner().unwrap()
}

#[allow(clippy::too_many_arguments)]
fn task(
    best_memory: Arc<Mutex<Vec<usize>>>,
    results: Arc<Mutex<MappedPaths>>,
    scheduler: Scheduler<Partitioner>,
    ntasks: i64,
    measure: Option<Vec<usize>>,
    timer: &Timer,
    probabilistic: Option<(&Accept, u64)>,
) {
    let _span = tracing::debug_span!("search task", ntasks).entered();

    tracing::debug!(
        "START: measure: {:?}; best_memory: {:?}",
        measure,
        best_memory.lock().unwrap()
    );

    let (mut new_results, this_best_mem) = if let Some(probabilistic) = probabilistic {
        do_probabilistic_search(
            scheduler.into_iter(),
            measure.map(|e| vec![e]),
            &best_memory,
            timer,
            probabilistic,
        )
    } else {
        do_search(scheduler.into_iter(), measure.map(|e| vec![e]), &best_memory, timer)
    };

    tracing::debug!("DONE: results {:?}; best_memory {:?}", new_results, this_best_mem,);

    if new_results.is_empty() {
        return;
    }

    let mut shared_best_mem = best_memory.lock().expect("failed to lock best_memory");
    let mut results = results.lock().expect("failed to lock results");

    for (time, (shared_mem, this_mem)) in
        shared_best_mem.iter_mut().zip(this_best_mem).enumerate()
    {
        if *shared_mem >= this_mem {
            *shared_mem = this_mem;
            // we cannot just unwrap, because when we do the
            // Step::Backward, we also update the best_memory for all
            // paths with a longer path length, but we might not have
            // collected a result for them (which is okay there)
            if let Some(mem) = new_results.remove(&time) {
                results.insert(time, mem);
            }
        }
    }
}

#[inline]
fn update(
    best_memory: &Arc<Mutex<Vec<usize>>>,
    this_best_mem: &mut [usize],
    update_counter: &mut usize,
    timer: &Timer,
) -> bool {
    if *update_counter == 1000 {
        best_memory
            .lock()
            .expect("failed to lock best_memory")
            .iter_mut()
            .zip(this_best_mem.iter_mut())
            .for_each(|(shared_mem, this_mem)| match (*this_mem).cmp(shared_mem) {
                Ordering::Greater => *this_mem = *shared_mem,
                Ordering::Less => *shared_mem = *this_mem,
                Ordering::Equal => {},
            });
        *update_counter = 1; // zero means we never updated; NEVER SET IT TO 0!!!!!!!!
        timer.finished()
    } else {
        *update_counter += 1;
        false
    }
}

fn do_search(
    mut scheduler: Sweep<Scheduler<Partition<Vec<usize>>>>,
    init_path: Option<OnePath>,
    best_memory: &Arc<Mutex<Vec<usize>>>,
    timer: &Timer,
) -> (MappedPaths, Vec<usize>) {
    let mut results = HashMap::new();
    let was_initialized = init_path.is_some();
    let mut current_path = init_path.unwrap_or_default();
    let mut this_best_mem =
        best_memory.lock().expect("failed to lock predicates").to_vec();
    let mut update_counter = 0;

    while let Some(step) = scheduler.next() {
        match step {
            Step::Forward(measure) => {
                if super::forward(
                    measure,
                    &mut scheduler,
                    &this_best_mem,
                    &mut current_path,
                ) {
                    break;
                }
            },
            Step::Backward(leaf) => {
                super::backward(
                    leaf,
                    &mut current_path,
                    &mut this_best_mem,
                    &mut results,
                );
            },
        }
        if update(best_memory, &mut this_best_mem, &mut update_counter, timer) {
            tracing::info!("timer: timeout");
            break;
        }
    }

    if update_counter == 0 && was_initialized {
        super::backward(
            Some(scheduler.current().space().max_memory()),
            &mut current_path,
            &mut this_best_mem,
            &mut results,
        )
    }

    (results, this_best_mem)
}

fn do_probabilistic_search(
    mut scheduler: Sweep<Scheduler<Partition<Vec<usize>>>>,
    init_path: Option<OnePath>,
    best_memory: &Arc<Mutex<Vec<usize>>>,
    timer: &Timer,
    (accept_func, seed): (&Accept, u64),
) -> (MappedPaths, Vec<usize>) {
    let mut results = HashMap::new();
    let was_initialized = init_path.is_some();
    let mut current_path = init_path.unwrap_or_default();
    let mut this_best_mem =
        best_memory.lock().expect("failed to lock best_memory").to_vec();
    let mut update_counter = 0;

    let mut rng = Pcg64::seed_from_u64(seed);
    let dist = Uniform::new(0., 1.);

    loop {
        let space = scheduler.current().space();
        let last_cur_mem = space.current_memory();
        let last_max_mem = space.max_memory();
        if let Some(step) = scheduler.next() {
            match step {
                Step::Forward(measure) => {
                    if super::probabilistic_forward(
                        measure,
                        &mut scheduler,
                        &this_best_mem,
                        &mut current_path,
                        last_cur_mem,
                        last_max_mem,
                        &mut rng,
                        &dist,
                        accept_func,
                    ) {
                        break;
                    }
                },
                Step::Backward(leaf) => {
                    super::backward(
                        leaf,
                        &mut current_path,
                        &mut this_best_mem,
                        &mut results,
                    );
                },
            }
            if update(best_memory, &mut this_best_mem, &mut update_counter, timer) {
                tracing::info!("timer: timeout");
                break;
            }
        } else {
            break;
        }
    }

    if update_counter == 0 && was_initialized {
        super::backward(
            Some(scheduler.current().space().max_memory()),
            &mut current_path,
            &mut this_best_mem,
            &mut results,
        )
    }

    (results, this_best_mem)
}
