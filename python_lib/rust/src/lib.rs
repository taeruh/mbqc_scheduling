use std::time::Duration;

use lib::interface::{self};
use pauli_tracker_pyo3::{frames::PartialOrderGraph, serde, transform, Module};
use probabilistic::AcceptFunc;
use pyo3::{exceptions::PyValueError, types::PyModule, PyAny, PyRef, PyResult, Python};

use self::probabilistic::AcceptFuncBase;

#[pyo3::pyclass(subclass)]
/// A list of neighbors for each node, describing the graph obtained from running the
/// stabilizer simulator (and transforming it into a graph).
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

    #[doc = transform!()]
    ///
    /// Returns:
    ///     list[list[int]]:
    #[allow(clippy::wrong_self_convention)]
    fn into_py_graph(&self) -> interface::SpacialGraph {
        self.0.clone()
    }
}

serde!(SpacialGraph);

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

    #[doc = transform!()]
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

serde!(Paths);

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
///     probabilistic (Optional[Tuple[AcceptFunc, Optional[int]]]): Whether to do the
///         search probabilistically or deterministically. If None, the search will be
///         deterministic. For larger problems, you will want to do it probabilistically,
///         with a relatively low accept rate, because otherwise it takes forever (scaling
///         is in the worst something between factorial and double exponential). The
///         second tuple element is an optional seed for the random number generator.
///         However, note that if multithreaded, i.e., `nthreads > 1`, fixing the seed
///         does not ensure reproducibibility (the threads communicate the results with
///         each other, and depending on that they adjust the search; this communication
///         is not deterministic (on this level here) since it depends on how the threads
///         are scheduled).
///     task_bound (int): The maximum number of tasks to start in the search, cf.
///         `nthreads`.
///
/// Returns:
///     Paths: A list of the optimal paths. Turn it into the corresponding Python object
///     via :meth:`Paths.into_py_paths`.
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
    spacial_graph: &SpacialGraph,
    // cf. https://github.com/PyO3/pyo3/issues/1444: when the PartialOrderGraph comes
    // directly from pauli_tracker, it errors because they are different python types
    // -> hacky fix: use pyany and check whether it is an instance of PartialOrderGraph
    // (then it comes from this lib) - then we can take it by reference, otherwise we try
    // to access the `into_py_graph` method and extract it (which clones)
    time_order: &PyAny,
    do_search: bool,
    timeout: Option<u32>,
    nthreads: u16,
    probabilistic: Option<(AcceptFunc, Option<u64>)>,
    task_bound: Option<u32>,
) -> PyResult<Paths> {
    // GIL problems ... (it completely locks the execution)
    if let Some((AcceptFunc(AcceptFuncBase::Custom(_)), _)) = probabilistic {
        if nthreads > 1 {
            return Err(PyValueError::new_err(
                "multi-threading with a custom Python callback is not supported",
            ));
        }
    }

    let mut _cloned: Vec<Vec<(usize, Vec<usize>)>>;
    let mut _by_ref: PyRef<'_, PartialOrderGraph>;
    let time_order = if time_order.is_instance_of::<PartialOrderGraph>() {
        _by_ref = time_order.extract()?;
        &_by_ref.0
    } else {
        eprintln!(
            r"warning: time_order is not the type `PartialOrderGraph' defined in the
    mbqc_scheduling package; trying to get the graph via 'into_py_graph' method"
        );
        _cloned = time_order.call_method("into_py_graph", (), None)?.extract()?;
        eprintln!(
            "    -> 'into_py_graph' succeeded, however, consider wrapping time_order
    into the correct type (to reduce potentially redundant cloning), e.g., if the object
    comes from the pauli_tracker package, replace `time_order` with
    `PartialOrderGraph(time_order.take_into_py_graphs())`"
        );
        &_cloned
    };

    Ok(Paths(interface::run(
        &spacial_graph.0,
        time_order,
        do_search,
        timeout.map(|t| Duration::from_secs(t.into())),
        nthreads,
        task_bound,
        probabilistic.map(|(func, seed)| (func.to_real(), seed)),
    )))
}

mod probabilistic;

#[pyo3::pymodule]
fn _lib(py: Python, module: &PyModule) -> PyResult<()> {
    let module = Module {
        pymodule: module,
        path: "mbqc_scheduling._lib".to_string(),
    };
    module.pymodule.add_class::<SpacialGraph>()?;
    module.pymodule.add_class::<PartialOrderGraph>()?;
    module.pymodule.add_class::<Paths>()?;
    module.pymodule.add_class::<Path>()?;
    module
        .pymodule
        .add_function(pyo3::wrap_pyfunction!(run, module.pymodule)?)?;
    probabilistic::add_module(py, &module)?;
    Ok(())
}
