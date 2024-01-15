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
        task_bound,
        probabilistic.then_some(AcceptFunc::BuiltinBasic),
        debug,
        (paths, &paths_format),
    )
    .expect("path search failed: ")
}
