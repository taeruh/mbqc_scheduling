use mbqc_scheduling::{
    interface,
    probabilistic::AcceptFunc,
};

mod cli;

fn main() {
    let (
        spacial_graph,
        spacial_graph_format,
        dependency_graph,
        dependency_graph_format,
        paths,
        paths_format,
        do_search,
        nthreads,
        probabilistic,
        task_bound,
        debug,
    ) = cli::parse();
    interface::run_serialized(
        (spacial_graph, &spacial_graph_format),
        (dependency_graph, &dependency_graph_format),
        do_search,
        nthreads,
        probabilistic.then_some(AcceptFunc::Standard),
        // TODO: make proper tests for this sanity test here (just checking here whether
        // there's no typo in AcceptFunc::CreateFunc)
        // probabilistic.then_some(create_accept_func()),
        task_bound,
        debug,
        (paths, &paths_format),
    )
    .expect("path search failed: ")
}

// cf. above
// fn create_accept_func() -> AcceptFunc {
//     AcceptFunc::CreateFunc {
//         weights: mbqc_scheduling::probabilistic::Weights {
//             last_max_mem: 1.,
//             last_cur_mem: 0.,
//             cur_mem: 1.,
//             num_measure_nodes: 1.,
//             num_total_nodes: 1.3e-1,
//         },
//         shifts: mbqc_scheduling::probabilistic::Shifts {
//             last_mem: 1.,
//             cur_mem: 1.,
//             time: 1e-3,
//             num_measure_nodes: 1.,
//             num_total_nodes: 1.3e-1,
//         },
//     }
// }
