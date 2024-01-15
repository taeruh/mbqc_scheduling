use cli::Args;
use mbqc_scheduling::{
    interface,
    probabilistic::AcceptFunc,
};

mod cli;

fn main() {
    let Args {
        spacial_graph,
        spacial_graph_format,
        dependency_graph,
        dependency_graph_format,
        paths,
        paths_format,
        search,
        nthreads,
        probabilistic,
        task_bound,
        debug,
    } = cli::parse();

    interface::run_serialized(
        (spacial_graph, &spacial_graph_format),
        (dependency_graph, &dependency_graph_format),
        search,
        nthreads,
        // probabilistic.then_some(AcceptFunc::Standard),
        // TODO: make proper tests for this sanity test here (just checking here whether
        // there's no typo in AcceptFunc::CreateFunc)
        task_bound,
        probabilistic.then_some(create_accept_func()),
        debug,
        (paths, &paths_format),
    )
    .expect("path search failed: ")
}

// cf. above
fn create_accept_func() -> AcceptFunc {
    AcceptFunc::ParametrizedBasic {
        weights: mbqc_scheduling::probabilistic::Weights {
            bound_best_mem: 1.,
            last_max_mem: 0.,
            last_cur_mem: 0.,
            cur_mem: 1.,
            num_measure_nodes: 1.,
            num_total_nodes: 8.5e-2,
        },
        shifts: mbqc_scheduling::probabilistic::Shifts {
            upper_mem: 1.,
            cur_mem: 1.,
            time: 1e-3,
            num_measure_nodes: 1.,
            num_total_nodes: 8.5e-2,
        },
    }
}
