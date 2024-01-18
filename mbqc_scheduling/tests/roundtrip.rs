include!("../../pauli_tracker/pauli_tracker/tests/roundtrips/tracking.rs");

use std::{
    cmp,
    mem,
    thread,
};

use hashbrown::HashSet;
use mbqc_scheduling::scheduler::{
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
    },
    Scheduler,
};
use proptest::strategy::Just;

type Edges = Vec<(usize, usize)>;

// with release mode we can push it a little bit further
// const MAX_NODES: usize = 11;
// const MAX_EDGES: usize = 60;
// const MAX_OPS: usize = 500;
const MAX_NODES: usize = 8;
const MAX_EDGES: usize = 40;
const MAX_OPS: usize = 200;
// const MAX_NODES: usize = 4;
// const MAX_EDGES: usize = 4;
// const MAX_OPS: usize = 10;
// const MAX_LAYERS: usize = MAX_ORDER_RELATIONS;
proptest! {
    #![proptest_config(Config {
        // cases: 10,
        // cases: 1,
        // see comment in super::tracking
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "regressions",
        ))),
        ..Default::default()
    })]
    #[test]
    #[ignore = "run proptests explicitly"]
    fn proptest(
        (frames, edges, num_nodes) in input(MAX_NODES, MAX_EDGES, MAX_OPS)
    ) {
        roundtrip(frames, edges, num_nodes);
    }
}

fn roundtrip(ops: Vec<Operation>, edges: Edges, num_nodes: usize) {
    // creating a DependencyGraph with proptest is not trivial, I don't see a way of how
    // to do it without either implemeting our own Strategies, which I don't want to do
    // at the moment, or by throwing a lot of data away; -> just use what we have done
    // for the tracking roundtrip (we don't merge the roundtrips, because this test here
    // cannot deal with many qubits)

    let ops = ops
        .into_iter()
        .filter_map(|op| match op {
            Operation::Measure(_) => None,
            Operation::NewQubit(_) => None,
            Operation::Rz(a) => match a % 4 {
                0 => Some(Operation::TeleportedX(a, a + 73)),
                1 => Some(Operation::TeleportedY(a, a + 73)),
                2 => Some(Operation::TeleportedZ(a, a + 73)),
                3 => None,
                _ => unreachable!(),
            },
            other => Some(other),
        })
        .collect::<Vec<_>>();

    // println!("{:?}", (num_nodes, edges.len(), ops.len()));

    let mut generator = Instructor::new(num_nodes, ops);
    let mut circuit = TrackedCircuit {
        circuit: DummyCircuit {},
        tracker: Frames::<Storage>::init(num_nodes),
        storage: Storage::default(),
    };
    let mut measurements = WhereMeasured(Vec::new());
    generator.apply(&mut circuit, &mut measurements);
    circuit.tracker.measure_and_store_all(&mut circuit.storage);

    if measurements.0.is_empty() {
        return;
    }

    let dependency_graph = induced_order::get_order(
        <Storage as Iterable>::iter_pairs(&circuit.storage),
        &measurements.0,
    );
    // let dependency_graph: Vec<Vec<(usize, Vec<usize>)>> = vec![
    //     dependency_graph
    //         .into_iter()
    //         .flatten()
    //         .map(|(n, _)| (n, vec![]))
    //         .collect(),
    // ];
    let _dependency_graph = dependency_graph.clone();

    // println!("{:?}", dependency_graph);
    // println!("{:?}", dependency_graph.len());
    // println!("{:?}", edges);

    let mut buffer = DependencyBuffer::new(num_nodes);
    let path_generator =
        PathGenerator::from_dependency_graph(dependency_graph, &mut buffer, None);
    let graph_buffer = GraphBuffer::new(&edges, num_nodes, None, true);
    let graph = Graph::new(&graph_buffer);
    #[allow(clippy::redundant_clone)]
    let _graph = graph.clone();

    let scheduler = Scheduler::new(path_generator.clone(), graph.clone());

    let mut all_paths = Vec::new();
    let mut this_path = Vec::new();

    for step in scheduler.clone() {
        match step {
            Step::Forward(measure_set) => this_path.push(measure_set),
            Step::Backward(leaf) => {
                if let Some(max_mem) = leaf {
                    all_paths.push((this_path.len(), max_mem, this_path.clone()));
                }
                this_path.pop();
            },
        }
    }

    let mut skipper_results = HashMap::new();
    let mut path = Vec::new();
    let mut best_memory = vec![num_nodes + 1; num_nodes + 1];
    let mut scheduler = scheduler.into_iter();

    while let Some(step) = scheduler.next() {
        match step {
            Step::Forward(mess) => {
                let current = scheduler.current();
                let minimum_time = if current.time().at_leaf().is_some() {
                    path.len() + 1
                } else if current.time().has_unmeasureable() {
                    path.len() + 3
                } else {
                    path.len() + 2
                };
                if current.space().max_memory() >= best_memory[minimum_time] {
                    if scheduler.skip_current().is_err() {
                        break;
                    }
                } else {
                    path.push(mess);
                }
            },
            Step::Backward(leaf) => {
                if let Some(mem) = leaf {
                    best_memory[path.len()] = mem;
                    for m in best_memory[path.len() + 1..].iter_mut() {
                        *m = cmp::min(*m, mem);
                    }
                    skipper_results.insert(path.len(), (mem, path.clone()));
                }
                path.pop();
            },
        }
    }

    let mut filtered_all_paths: HashMap<usize, (usize, Vec<Vec<usize>>)> = HashMap::new();
    for (len, mem, path) in all_paths.iter() {
        if let Some(optimal) = filtered_all_paths.get_mut(len) {
            if optimal.0 > *mem {
                *optimal = (*mem, path.to_vec());
            }
        } else {
            filtered_all_paths.insert(*len, (*mem, path.to_vec()));
        }
    }

    // filter out paths that are longer than other paths with the same memory
    fn final_filter(
        num_nodes: usize,
        paths: HashMap<usize, (usize, Vec<Vec<usize>>)>,
    ) -> HashMap<usize, (usize, Vec<Vec<usize>>)> {
        let mut best_memory = vec![num_nodes + 1; num_nodes + 1];
        let mut res: HashMap<usize, (usize, Vec<Vec<usize>>)> = HashMap::new();
        for i in 0..best_memory.len() {
            if let Some((mem, _)) = paths.get(&i) {
                let m = best_memory[i];
                if *mem < m {
                    res.insert(i, paths.get(&i).unwrap().clone());
                    for m in best_memory[i..].iter_mut() {
                        *m = *mem;
                    }
                }
            }
        }
        res
    }

    let filtered_skipper_results = final_filter(num_nodes, skipper_results.clone());
    let filtered_filtered_all_paths = final_filter(num_nodes, filtered_all_paths.clone());

    // println!("{:?}", all_results);
    // println!("{:?}", optimal_paths);
    // println!("{:?}", skipper_results);

    assert_eq!(
        filtered_skipper_results, filtered_filtered_all_paths,
        "\n{all_paths:?}\n{filtered_filtered_all_paths:?}\n{filtered_skipper_results:?}"
    );

    let splitted_instructions = split_instructions(path_generator, 4);

    struct InstructedSweep<'l, I: Iterator<Item = TimeStep>> {
        current: Graph<'l>,
        stack: Vec<Graph<'l>>,
        instructions: I,
    }
    fn into_instructed_iterator<T: IntoIterator<Item = TimeStep>>(
        graph: Graph,
        instructions: T,
    ) -> InstructedSweep<T::IntoIter> {
        InstructedSweep {
            current: graph,
            stack: Vec::new(),
            instructions: instructions.into_iter(),
        }
    }
    enum Next {
        Mess(Vec<usize>),
        Mem(Option<usize>),
    }
    impl<'l, T: Iterator<Item = TimeStep>> Iterator for InstructedSweep<'l, T> {
        type Item = Next;
        fn next(&mut self) -> Option<Self::Item> {
            match self.instructions.next()? {
                Step::Forward(mess) => {
                    let new = self.current.focus(&mess).unwrap();
                    self.stack.push(mem::replace(&mut self.current, new));
                    Some(Next::Mess(mess))
                },
                Step::Backward(at_leaf) => {
                    let res = at_leaf.and(Some(self.current.max_memory()));
                    self.current = self.stack.pop().unwrap();
                    Some(Next::Mem(res))
                },
            }
        }
    }

    let merged_results = thread::scope(move |s| {
        let mut splitted_results = Vec::new();

        for split in splitted_instructions {
            let graph = graph.clone();
            splitted_results.push(s.spawn(move || {
                // println!("{:?}", split.len());
                let mut path = Vec::new();
                let mut results = Vec::new();
                for step in into_instructed_iterator(graph, split) {
                    match step {
                        Next::Mess(mess) => {
                            path.push(mess);
                        },
                        Next::Mem(mem) => {
                            if let Some(mem) = mem {
                                results.push((path.len(), mem, path.clone()));
                            }
                            path.pop();
                        },
                    }
                }
                results
            }));
        }

        let mut merged_results =
            HashSet::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for result in splitted_results {
            for r in result.join().unwrap() {
                assert!(merged_results.insert(r));
            }
        }
        merged_results
    });

    // cannot not directly use a HashSet above, because than the optimal_paths are not
    // deterministic; we don't use HashSet::from_iter, because we want to additionally
    // check that there are no duplicate paths
    let mut all_results_as_set =
        HashSet::with_hasher(BuildHasherDefault::<FxHasher>::default());
    for r in all_paths {
        assert!(all_results_as_set.insert(r));
    }
    assert_eq!(all_results_as_set, merged_results);

    // let mut sorted_final = filtered_skipper_results.into_iter().collect::<Vec<_>>();
    // sorted_final.sort_by_key(|(len, _)| *len);
    // // let mut sorted_intermediate = skipper_results.into_iter().collect::<Vec<_>>();
    // // sorted_intermediate.sort_by_key(|(len, _)| *len);
    // // if sorted_final != sorted_intermediate {
    // //     println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    // // }
    // println!("{:?}", _graph);
    // println!("{:?}", _dependency_graph);
    // // for r in sorted_intermediate.iter() {
    // //     println!("{:?}", r);
    // // }
    // for r in sorted_final.iter() {
    //     println!("{:?}", r);
    // }
    // println!();

    //
}

type TimeStep = Step<Vec<usize>, Option<()>>;

fn split_instructions(
    time: PathGenerator<Partitioner>,
    num_tasks: usize,
) -> Vec<Vec<TimeStep>> {
    let mut total_num_paths = 0;
    let instructions = time
        .into_iter()
        .inspect(|e| {
            if let Step::Backward(Some(())) = e {
                total_num_paths += 1;
            };
        })
        .collect::<Vec<TimeStep>>();

    if total_num_paths < num_tasks {
        return vec![instructions];
    }

    let mut paths_per_job = total_num_paths / num_tasks;
    let num_normal_tasks = num_tasks - (total_num_paths - num_tasks * paths_per_job);
    let mut res = Vec::new();
    let mut task = Vec::new();
    let mut paths_in_task = 0;
    let mut init = Vec::new();
    let mut num_done_tasks = 0;
    let mut instructions = instructions.into_iter();

    while let Some(step) = instructions.next() {
        match step {
            ref step @ Step::Backward(at_leaf) => {
                task.push(step.clone());
                init.pop();

                if at_leaf.is_none() {
                    continue;
                }

                paths_in_task += 1;
                if paths_in_task != paths_per_job {
                    continue;
                }

                paths_in_task = 0;
                for step in instructions.by_ref() {
                    match step {
                        Step::Backward(_) => {
                            init.pop();
                        },
                        step => {
                            init.push(step);
                            break;
                        },
                    }
                }

                res.push(mem::replace(&mut task, init.clone()));

                num_done_tasks += 1;
                if num_done_tasks == num_normal_tasks {
                    paths_per_job += 1;
                }
            },
            step => {
                task.push(step.clone());
                init.push(step);
            },
        }
    }
    res
}

fn input(
    max_nodes: usize,
    max_edges: usize,
    max_ops: usize,
) -> impl Strategy<Value = (Vec<Operation>, Edges, usize)> {
    (1..max_nodes, 0..max_edges, 0..max_ops).prop_flat_map(
        |(num_nodes, num_edges, num_ops)| {
            (
                fixed_num_vec_operation(num_ops),
                edges(num_edges, num_nodes),
                Just(num_nodes),
            )
        },
    )
}

fn edges(num_edges: usize, num_nodes: usize) -> impl Strategy<Value = Edges> {
    let mut res = Vec::new();
    for _ in 0..num_edges {
        res.push((0..num_nodes, 0..num_nodes));
    }
    res
}
