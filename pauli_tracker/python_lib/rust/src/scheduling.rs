use std::time::Duration;

use mbqc_scheduling::interface::{self};
use pyo3::{exceptions::PyValueError, PyResult, Python};

use crate::{
    frames::PartialOrderGraph,
    impl_helper::{doc, serialization},
    Module,
};

mod probabilistic;
use probabilistic::AcceptFunc;

use self::probabilistic::AcceptFuncBase;

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

#[pyo3::pyclass(subclass)]
/// Opaque Rust object. The information returned from the scheduling algorithm (`run`)
/// describing valid initalization-measurement paths.
#[derive(Clone)]
pub struct Paths(pub Vec<interface::Path>);

#[pyo3::pymethods]
impl Paths {
    #[new]
    fn __new__(paths: Vec<Path>) -> Self {
        Self(
            paths
                .into_iter()
                .map(|Path { time, space, steps }| interface::Path { time, space, steps })
                .collect(),
        )
    }

    /// Create a new Paths object.
    ///
    /// Args:
    ///    paths (list[Path]): The paths to create the Paths object from.
    ///
    /// Returns:
    ///     Paths:
    #[pyo3(text_signature = "(self, paths)")]
    fn __init__(&self, _paths: Vec<Path>) {}

    #[doc = doc::transform!()]
    ///
    /// Returns:
    ///    list[Path]:
    #[allow(clippy::wrong_self_convention)]
    fn into_py_paths(&self) -> Vec<Path> {
        self.0
            .clone()
            .into_iter()
            .map(|interface::Path { time, space, steps }| Path { time, space, steps })
            .collect()
    }
}

serialization::serde!(Paths);

#[pyo3::pyclass(subclass)]
/// The information describing a valid initalization-measurement path.
#[derive(Clone)]
pub struct Path {
    #[pyo3(get)]
    /// The number of (parallel) :attr:`steps`.
    pub time: usize,
    #[pyo3(get)]
    /// The number of required qubits.
    pub space: usize,
    #[pyo3(get)]
    /// The initialization-measurement steps.
    pub steps: Vec<Vec<usize>>,
}

#[pyo3::pymethods]
impl Path {
    #[new]
    fn __new__(time: usize, space: usize, steps: Vec<Vec<usize>>) -> Self {
        Self { time, space, steps }
    }

    /// Create a new Path object.
    ///
    /// Args:
    ///     time (int): The number of (parallel) steps (should be the length of
    ///         :attr:`steps`).
    ///     space (int): The number of required qubits.
    ///     steps (list[list[int]]): The initialization-measurement steps.
    ///
    /// Returns:
    ///     Path:
    #[pyo3(text_signature = "(self, time, space, steps)")]
    fn __init__(&self, _time: usize, _space: usize, _steps: Vec<Vec<usize>>) {}
}

/// Search for optimal initalization-measurement paths.
///
/// Args:
///     spacial_graph (SpacialGraph): The spacial graph.
///     time_order (PartialOrderGraph): The dependency graph on the measurements.
///     do_search (bool): Whether to search for all best paths or just take the first
///         one, which is the time optimal path. Searching for all best paths may take
///         some time ...
///     timeout (Optional[int]): A timeout for the search. You'll probably want to set
///         this, because if the run is cancelled by some other reason, the results are
///         generally lost, but when the run cancelled because of a timeout, the function
///         returns as normally with the results obtained so far. However, note that is
///         timeout is too short, i.e., shorter than how long it would take to get the
///         first path (which depends potentially `probabilistic`), then the function will
///         return an empty list.
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
///
/// Returns:
///     Paths: A list of the optimal paths. Turn it into the corresponding Python object
///     via :meth:`Paths.into_py_paths`.
///
/// When setting the variable MBQC_SCHEDULING_DEBUG to something, the search will print
/// some more or less useful debug information (if multithreaded); this is *unstable*
/// though.
#[pyo3::pyfunction]
#[pyo3(signature = (
    spacial_graph,
    time_order,
    do_search=false,
    timeout=None,
    nthreads=1,
    probabilistic=None,
    task_bound=None,
))]
#[allow(clippy::too_many_arguments)]
fn run(
    spacial_graph: SpacialGraph,
    time_order: PartialOrderGraph,
    do_search: bool,
    timeout: Option<u32>,
    nthreads: u16,
    probabilistic: Option<AcceptFunc>,
    task_bound: Option<u32>,
) -> PyResult<Paths> {
    // GIL problems ... (it completely locks the execution)
    if let Some(AcceptFunc(AcceptFuncBase::Custom(_))) = probabilistic {
        if nthreads > 1 {
            return Err(PyValueError::new_err(
                "multi-threading with a custom Python callback is not supported",
            ));
        }
    }
    Ok(Paths(interface::run(
        spacial_graph.0,
        time_order.0,
        do_search,
        timeout.map(|t| Duration::from_secs(t.into())),
        nthreads,
        task_bound,
        probabilistic.map(|e| e.to_real()),
    )))
}

pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "scheduling", parent_module.path.clone())?;
    module.pymodule.add_class::<SpacialGraph>()?;
    module.pymodule.add_class::<Paths>()?;
    module.pymodule.add_class::<Path>()?;
    module
        .pymodule
        .add_function(pyo3::wrap_pyfunction!(run, module.pymodule)?)?;
    probabilistic::add_module(py, &module)?;
    parent_module.add_submodule(py, module)?;
    Ok(())
}
