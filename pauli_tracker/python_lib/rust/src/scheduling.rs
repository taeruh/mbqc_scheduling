use mbqc_scheduling::{
    interface::{
        self,
        Paths,
    },
    probabilistic,
};
use pyo3::{
    exceptions::PyRuntimeError,
    PyObject,
    PyResult,
    Python,
};

use crate::{
    frames::DependencyGraph,
    impl_helper::{
        doc,
        serialization,
    },
    Module,
};

/// A list of neighbors for each node, describing the graph obtained from running the
/// stabilizer simulator (and transforming it into a graph).
#[pyo3::pyclass(subclass)]
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
    fn __init__(&mut self, _graph: interface::SpacialGraph) {}

    #[doc = doc::transform!()]
    ///
    /// Returns:
    ///     list[list[int]]:
    #[allow(clippy::wrong_self_convention)]
    fn into_py_graph(&self) -> interface::SpacialGraph {
        self.0.clone()
    }
}

#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct Weights(probabilistic::Weights);

#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct Shifts(probabilistic::Shifts);

#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct CreateFuncParameters {
    pub weights: Weights,
    pub shifts: Shifts,
}

#[pyo3::pyclass]
#[derive(Clone)]
pub enum AcceptFuncKind {
    Standard,
    CreateFunc,
    Custom,
}

#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct AcceptFunc {
    pub kind: AcceptFuncKind,
    pub create_func_parameters: Option<CreateFuncParameters>,
    pub custom_func: Option<PyObject>,
}

#[pyo3::pymethods]
impl Weights {
    #[new]
    fn __new__(
        last_max_mem: f64,
        last_cur_mem: f64,
        cur_mem: f64,
        num_measure_nodes: f64,
        num_total_nodes: f64,
    ) -> Self {
        Self(probabilistic::Weights {
            last_max_mem,
            last_cur_mem,
            cur_mem,
            num_measure_nodes,
            num_total_nodes,
        })
    }
}

#[pyo3::pymethods]
impl Shifts {
    #[new]
    fn __new__(
        last_mem: f64,
        cur_mem: f64,
        time: f64,
        num_measure_nodes: f64,
        num_total_nodes: f64,
    ) -> Self {
        Self(probabilistic::Shifts {
            last_mem,
            cur_mem,
            time,
            num_measure_nodes,
            num_total_nodes,
        })
    }
}

#[pyo3::pymethods]
impl CreateFuncParameters {
    #[new]
    fn __new__(weights: Weights, shifts: Shifts) -> Self {
        Self { weights, shifts }
    }
}

#[pyo3::pymethods]
impl AcceptFunc {
    #[new]
    fn __new__(
        kind: AcceptFuncKind,
        create_func_parameters: Option<CreateFuncParameters>,
        custom_func: Option<PyObject>,
    ) -> Self {
        Self {
            kind,
            create_func_parameters,
            custom_func,
        }
    }
}

impl AcceptFunc {
    fn to_real(&self) -> Result<probabilistic::AcceptFunc, String> {
        Ok(match self.kind {
            AcceptFuncKind::Standard => probabilistic::AcceptFunc::Standard,
            AcceptFuncKind::CreateFunc => probabilistic::AcceptFunc::CreateFunc {
                weights: self.create_func_parameters.clone().ok_or("Foo")?.weights.0,
                shifts: self
                    .create_func_parameters
                    .clone()
                    .ok_or("Bar")?
                    .shifts
                    .0
                    .clone(),
            },
            AcceptFuncKind::Custom => match self.custom_func.clone() {
                None => return Err("Custom accept function not set".to_string()),
                Some(obj) => probabilistic::AcceptFunc::Custom(Box::new(
                    move |last_max_mem: f64,
                          last_cur_mem: f64,
                          cur_mem: f64,
                          num_remaining_nodes: f64,
                          num_total_nodes: f64|
                          -> f64 {
                        Python::with_gil(|py| {
                            obj.call(
                                py,
                                (
                                    last_max_mem,
                                    last_cur_mem,
                                    cur_mem,
                                    num_remaining_nodes,
                                    num_total_nodes,
                                ),
                                None,
                            )
                            .unwrap()
                            .extract(py)
                            .unwrap()
                        })
                    },
                )),
            },
        })
    }
}

serialization::serde!(SpacialGraph);

// copilot: make a python docstring of the above

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
    do_search,
    nthreads,
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
) -> PyResult<Paths> {
    interface::run(
        spacial_graph.0,
        dependency_graph.0,
        do_search,
        nthreads,
        probabilistic.map(|e| e.to_real().unwrap()),
        task_bound,
        debug,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "scheduling", parent_module.path.clone())?;

    module.add_class::<SpacialGraph>()?;
    module.add_class::<AcceptFunc>()?;
    module.add_class::<AcceptFuncKind>()?;
    module.add_class::<Weights>()?;
    module.add_class::<Shifts>()?;
    module.add_class::<CreateFuncParameters>()?;
    module.add_function(pyo3::wrap_pyfunction!(run, module.pymodule)?)?;

    parent_module.add_submodule(py, module)?;
    Ok(())
}
