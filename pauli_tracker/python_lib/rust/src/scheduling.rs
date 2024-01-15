use mbqc_scheduling::interface::{
    self,
    Paths,
};
use pyo3::{
    PyResult,
    Python,
};

use crate::{
    frames::DependencyGraph,
    impl_helper::{
        doc,
        serialization,
    },
    probabilistic::AcceptFunc,
    Module,
};

#[pyo3::pyclass(subclass)]
/// A list of neighbors for each node, describing the graph obtained from running the
/// stabilizer simulator (and transforming it into a graph).
#[derive(Clone)]
pub struct SpacialGraph(pub interface::SpacialGraph);

#[pyo3::pymethods]
impl SpacialGraph {
    #[new]
    fn __new__(graph: interface::SpacialGraph) -> Self {
        Self(graph)
    }

    /// Create a new SpacialGraph.
    ///
    /// Args:
    ///     graph (list[list[int]]): The graph to create the SpacialGraph from.
    ///
    /// Returns:
    ///     SpacialGraph:
    #[pyo3(text_signature = "(self, graph)")]
    fn __init__(&self, _graph: interface::SpacialGraph) {}

    #[doc = doc::transform!()]
    ///
    /// Returns:
    ///     list[list[int]]:
    #[allow(clippy::wrong_self_convention)]
    fn into_py_graph(&self) -> interface::SpacialGraph {
        self.0.clone()
    }
}

serialization::serde!(SpacialGraph);

/// Search for optimal initalization-measurement paths.
///
/// Args:
///     spacial_graph (SpacialGraph): The spacial graph.
///     dependency_graph (DependencyGraph): The dependency graph on the measurements.
///     do_search (bool): Whether to search for all best paths or just take the first
///         one, which is the time optimal path. Searching for all best paths may take
///         some time ...
///     nthreads (int): The number of threads to use for the search. If `nthreads` is
///         below 3, it will not multithread. Otherwise it will start a threadpool
///         (where one thread is used to manage shared data). The tasks for the
///         threadpool are all the possible focused Scheduler sweeps after doing one
///         initial focus, cf. source code .... The number of those task scales
///         exponentially with the number of bits in the first layer of the dependency
///         graph. Use the `task_bound` option to limit the number of these tasks (but
///         the then last task may take some time because it does all remaining tasks).
///     probabilistic (Optional[AcceptFunc]): Whether to do the search probabilistically
///         or deterministically. If None, the search will be deterministic. For larger
///         problems, you will want to do it probabilistically, with a relatively low
///         accept rate, because otherwise it takes forever (scaling is in the worst
///         something between factorial and double exponential).
///     task_bound (int): The maximum number of tasks to start in the search, cf.
///         `nthreads`.
///     debug (bool): Whether to print some more or less useful information while
///         multithreading.
///
/// Returns:
///     list[tuple[int, tuple[int, list[list[int]]]]]: A list of the optimal paths. In
///     the outer tuple, the first entry is the number of time steps, and the second
///     tuple contains the total number of required qubits and the list of qubits that
///     can be measured in parallel at each time step.
#[pyo3::pyfunction]
#[pyo3(signature = (
    spacial_graph,
    dependency_graph,
    do_search=false,
    nthreads=1,
    probabilistic=None,
    task_bound=None,
    debug=false,
))]
fn run(
    spacial_graph: SpacialGraph,
    dependency_graph: DependencyGraph,
    do_search: bool,
    nthreads: u16,
    probabilistic: Option<AcceptFunc>,
    task_bound: Option<u32>,
    debug: bool,
) -> Paths {
    interface::run(
        spacial_graph.0,
        dependency_graph.0,
        do_search,
        nthreads,
        task_bound,
        probabilistic.map(|e| e.to_real()),
        debug,
    )
}

pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "scheduling", parent_module.path.clone())?;

    module.add_class::<SpacialGraph>()?;
    module.add_function(pyo3::wrap_pyfunction!(run, module.pymodule)?)?;

    parent_module.add_submodule(py, module)?;
    Ok(())
}
