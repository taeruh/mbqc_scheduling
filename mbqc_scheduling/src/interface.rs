use std::path;

use anyhow::Result;
use pauli_tracker::tracker::frames::induced_order::PartialOrderGraph;
use serde::{Deserialize, Serialize};

pub use crate::search::SpacialGraph;
use crate::{probabilistic::AcceptFunc, search};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    pub time: usize,
    pub space: usize,
    pub steps: Vec<Vec<usize>>,
}

/// Searching for optimal initialization-measuremnt paths.
///
/// - `spacial_graph` is a list of neighbors for each node, describing the graph
/// obtained from running the stabilizer simulator (and transforming it into a graph).
/// - `dependency_graph` is the output obtained from the pauli tracker, describing the
/// partial ordering of the measurements in time.
/// - `do_search` is a flag that determines whether to search for all best paths or just
/// take the first one, which is the time optimal path. Searching for all best paths may
/// take some time ...
/// - `nthreads` is the number of threads to use for the search. If `nthreads` is below
/// 2, it will not multithread. Otherwise it will start a threadpool (where one thread
/// is used to manage shared data). The tasks for the threadpool are all the possible
/// focused Scheduler sweeps after doing one initial focus, cf. source code .... The
/// number of those task scales exponentially with the number of bits in the first layer
/// of the dependency graph. Use the `task_bound` option to limit the number of these
/// tasks (but the then last task may take some time because it does all remaining
/// tasks).
/// - `task_bound` is the maximum number of tasks to start in the search, cf.
/// `nthreads`.
/// - `probabilistic` specifies whether the search should be overlayed with an
/// [AcceptFunc] that specifies the probability to accept a step in the path search. If
/// None, the search will be deterministically. For larger problems, you will want to do
/// it probabilistically, with a relatively low accept rate, because otherwise it takes
/// forever (scaling is in the worst case something between factorial and double
/// exponential).
/// - `debug` is a flag that determines whether to print some more or less useful
/// information when multithreading ...
pub fn run(
    spacial_graph: SpacialGraph,
    time_ordering: PartialOrderGraph,
    do_search: bool,
    nthreads: u16,
    task_bound: Option<u32>,
    probabilistic: Option<AcceptFunc>,
    debug: bool,
) -> Vec<Path> {
    if !do_search {
        search::get_time_optimal(spacial_graph, time_ordering)
    } else {
        search::search(
            spacial_graph,
            time_ordering,
            nthreads,
            probabilistic.map(AcceptFunc::get_accept_func),
            task_bound.map(|b| b.into()).unwrap_or(10000),
            debug,
        )
    }
}

use utils::serialization::Dynamic;

/// Same as [run], but with file paths to the input and output data.
#[allow(clippy::too_many_arguments)]
pub fn run_serialized(
    spacial_graph: (impl AsRef<path::Path>, &str),
    dependency_graph: (impl AsRef<path::Path>, &str),
    do_search: bool,
    nthreads: u16,
    task_bound: Option<u32>,
    probablistic: Option<AcceptFunc>,
    debug: bool,
    paths: (impl AsRef<path::Path>, &str),
) -> Result<()> {
    let spacial_graph = Dynamic::try_from(spacial_graph.1)?.read_file(spacial_graph.0)?;
    let dependency_graph =
        Dynamic::try_from(dependency_graph.1)?.read_file(dependency_graph.0)?;
    Dynamic::try_from(paths.1)?
        .write_file(
            paths.0,
            &run(
                spacial_graph,
                dependency_graph,
                do_search,
                nthreads,
                task_bound,
                probablistic,
                debug,
            ),
        )
        .map_err(Into::into)
}
