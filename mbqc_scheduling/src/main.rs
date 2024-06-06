use std::time::Duration;

use cli::Args;
use mbqc_scheduling::{interface, probabilistic::AcceptFunc};

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
        timeout,
        nthreads,
        probabilistic,
        task_bound,
    } = cli::parse();
    tracing_subscriber::fmt::init();
    interface::run_serialized(
        (spacial_graph, &spacial_graph_format),
        (dependency_graph, &dependency_graph_format),
        search,
        timeout.map(|t| Duration::from_secs(t.into())),
        nthreads,
        task_bound,
        probabilistic.then_some(AcceptFunc::BuiltinHeavyside),
        (paths, &paths_format),
    )
    .expect("path search failed")
}
