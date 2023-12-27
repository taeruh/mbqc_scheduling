use mbqc_scheduling::interface;

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
        task_bound,
    ) = cli::parse();
    interface::run_serialized(
        (spacial_graph, &spacial_graph_format),
        (dependency_graph, &dependency_graph_format),
        do_search,
        nthreads,
        task_bound,
        (paths, &paths_format),
    )
    .expect("path search failed: ")
}
