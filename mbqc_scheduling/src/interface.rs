/*!
Main interface to run the search algorithms.
*/

use std::{error, fmt, fs, fs::File, io, path, time::Duration};

use pauli_tracker::tracker::frames::induced_order::PartialOrderGraph;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{probabilistic::AcceptFunc, scheduler::space::SpacialGraph, search};
pub use crate::{
    scheduler::{space::RefSpacialGraph, time::RefPartialOrderGraph},
    search::Steps,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Description of a measurement pattern/path/schedule
// Pattern would be a better name, however, we also use Path in the python wrapper, which
// is published, so we wait until we have to do a breaking change for other reasons there.
pub struct Path {
    /// The time cost, i.e., the number of parallel measurement `steps` (it's just
    /// `steps.len()`).
    pub time: usize,
    /// The space cost, i.e., the maximum number of qubits that have been in memory at a
    /// certain point in time.
    pub space: usize,
    /// The measurement pattern, consisting of a list of parallel measurement steps.
    pub steps: Steps,
}

/// Searching for optimal initialization-measurement [Path]s.
///
/// # Arguments
///
/// * `spacial_graph` - A list of neighbors for each node, describing the graph
/// obtained from running the stabilizer simulator (and transforming it into a graph).
/// * `dependency_graph` - The output obtained from the pauli tracker, describing the
/// partial ordering of the measurements in time.
/// * `do_search` - A flag that determines whether to search for all best paths or just
/// take the first one, which is the time optimal path. Searching for all best paths may
/// take some time ...
/// * `timeout` - A timeout for the search. You'll probably want to set this, because if
/// the run is cancelled by some other reason, the results are generally lost, but when
/// the run cancelled because of a timeout, the function returns as normally with the
/// results obtained so far. However, note that is timeout is too short, i.e., shorter
/// than how long it would take to get the first path (which depends potentially
/// `probabilistic`), then the function will return an empty list.
/// * `nthreads` - the number of threads to use for the search. If `nthreads` is below
/// 2, it will not multithread. Otherwise it will start a threadpool (where one thread
/// is used to manage shared data). The tasks for the threadpool are all the possible
/// focused Scheduler sweeps after doing one initial focus, cf. source code .... The
/// number of those task scales exponentially with the number of bits in the first layer
/// of the dependency graph. Use the `task_bound` option to limit the number of these
/// tasks (but the then last task may take some time because it does all remaining
/// tasks).
/// * `task_bound` - The maximum number of tasks to start in the search, cf.
/// `nthreads`.
/// * `probabilistic` - Specifies whether the search should be overlayed with an
/// [AcceptFunc] that specifies the probability to accept a step in the path search. If
/// None, the search will be deterministically. For larger problems, you will want to do
/// it probabilistically, with a relatively low accept rate, because otherwise it takes
/// forever (scaling is in the worst case something between factorial and double
/// exponential). The second tuple element is an optional seed for the random number
/// generator. However, note that if multithreaded, i.e., `nthreads > 1`, fixing the seed
/// does not ensure reproducibibility (the threads communicate the results with each
/// other, and depending on that they adjust the search; this communication is not
/// deterministic (on this level here) since it depends on how the threads are scheduled).
///
/// Note that the algorithm always first tries the more time optimal patterns, however,
/// whether they are accepted can be controlled with the `probabilistic` accept function.
pub fn run(
    spacial_graph: RefSpacialGraph,
    time_ordering: RefPartialOrderGraph,
    do_search: bool,
    timeout: Option<Duration>,
    nthreads: u16,
    task_bound: Option<u32>,
    probabilistic: Option<(AcceptFunc, Option<u64>)>,
) -> Vec<Path> {
    if !do_search {
        search::get_time_optimal(spacial_graph, time_ordering)
    } else {
        search::search(
            spacial_graph,
            time_ordering,
            timeout,
            nthreads,
            probabilistic.map(|(func, seed)| (func.get_accept_func(), seed)),
            task_bound.map(|b| b.into()).unwrap_or(100000),
        )
    }
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Same as [run], but with file paths to the input and output data.
#[allow(clippy::too_many_arguments)]
pub fn run_serialized(
    spacial_graph: (impl AsRef<path::Path>, &str),
    dependency_graph: (impl AsRef<path::Path>, &str),
    do_search: bool,
    timeout: Option<Duration>,
    nthreads: u16,
    task_bound: Option<u32>,
    probablistic: Option<AcceptFunc>,
    paths: (impl AsRef<path::Path>, &str),
) -> Result<()> {
    let spacial_graph: SpacialGraph =
        deserialize_from_file(spacial_graph.0, spacial_graph.1)?;
    let dependency_graph: PartialOrderGraph =
        deserialize_from_file(dependency_graph.0, dependency_graph.1)?;
    serialize_to_file(
        paths.0,
        &run(
            &spacial_graph,
            &dependency_graph,
            do_search,
            timeout,
            nthreads,
            task_bound,
            probablistic.map(|func| (func, None)),
        ),
        paths.1,
    )
}

fn open(path: impl AsRef<path::Path>) -> io::Result<File> {
    File::open(path)
}

fn create(path: impl AsRef<path::Path>) -> io::Result<File> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)
}

#[derive(Debug)]
struct UnknownFormat(String);

impl fmt::Display for UnknownFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown format: {}", self.0)
    }
}

impl error::Error for UnknownFormat {}

fn serialize_to_file<T: Serialize, P: AsRef<path::Path>>(
    path: P,
    value: &T,
    format: &str,
) -> Result<()> {
    match format {
        "serde_json" => serde_json::to_writer(create(path)?, value)?,
        "bincode" => bincode::serialize_into(create(path)?, value)?,
        _ => return Err(UnknownFormat(format.to_owned()).into()),
    };
    Ok(())
}

fn deserialize_from_file<T: DeserializeOwned, P: AsRef<path::Path>>(
    path: P,
    format: &str,
) -> Result<T> {
    Ok(match format {
        "serde_json" => serde_json::from_reader(open(path)?)?,
        "bincode" => bincode::deserialize_from(open(path)?)?,
        _ => return Err(UnknownFormat(format.to_owned()).into()),
    })
}
