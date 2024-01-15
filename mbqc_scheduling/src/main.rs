use cli::Args;
use mbqc_scheduling::{
    interface,
    probabilistic::AcceptFunc,
};
use utils::serialization::Dynamic;

mod cli;

fn transform() {
    use pauli_tracker::{
        collection::{
            Iterable,
            Map,
        },
        pauli::PauliStack,
        tracker::frames::{
            dependency_graph,
            Frames,
        },
    };
    let serde = Dynamic::SerdeJson;
    let frames: Frames<Map<PauliStack<Vec<bool>>>> = serde
        .read_file("../test_files/cluster/fourier_5_ooooo_frames.json")
        .unwrap();
    serde
        .write_file(
            "../test_files/fourier_5o_dependency.json",
            &dependency_graph::create_dependency_graph(
                frames.as_storage().iter_pairs(),
                &[4, 5, 6, 7, 8, 9, 10, 11, 3, 13, 14, 15, 16, 17, 2, 19, 20, 21, 1, 23],
            ),
        )
        .unwrap();
}

fn main() {
    transform()

    // let Args {
    //     spacial_graph,
    //     spacial_graph_format,
    //     dependency_graph,
    //     dependency_graph_format,
    //     paths,
    //     paths_format,
    //     search,
    //     nthreads,
    //     probabilistic,
    //     task_bound,
    //     debug,
    // } = cli::parse();

    // interface::run_serialized(
    //     (spacial_graph, &spacial_graph_format),
    //     (dependency_graph, &dependency_graph_format),
    //     search,
    //     nthreads,
    //     task_bound,
    //     probabilistic.then_some(AcceptFunc::BuiltinBasic),
    //     debug,
    //     (paths, &paths_format),
    // )
    // .expect("path search failed: ")
}
