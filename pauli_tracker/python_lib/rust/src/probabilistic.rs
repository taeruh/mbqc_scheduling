use mbqc_scheduling::probabilistic;
use pyo3::{
    exceptions::PyValueError,
    PyObject,
    PyResult,
    Python,
};

use crate::Module;

#[pyo3::pyclass(subclass)]
/// Compare the corresponding documentation in the `mbqc_scheduling crate
/// <https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_.
#[derive(Clone)]
pub struct Weights(probabilistic::Weights);

#[pyo3::pyclass(subclass)]
/// Compare the corresponding documentation in the `mbqc_scheduling crate
/// <https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_.
#[derive(Clone)]
pub struct Shifts(probabilistic::Shifts);

// note that the original probabilistic::AcceptFunc cannot implement Clone
#[derive(Clone)]
enum AcceptFuncBase {
    BuiltinBasic,
    ParametrizedBasic { weights: Weights, shifts: Shifts },
    Custom(PyObject),
}

#[pyo3::pyclass(subclass)]
/// Compare the corresponding documentation in the `mbqc_scheduling crate
/// <https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_.
#[derive(Clone)]
pub struct AcceptFunc(AcceptFuncBase);

#[pyo3::pymethods]
impl Weights {
    #[new]
    fn __new__(
        bound_best_mem: f64,
        last_max_mem: f64,
        last_cur_mem: f64,
        cur_mem: f64,
        num_measure_nodes: f64,
        num_total_nodes: f64,
    ) -> Self {
        Self(probabilistic::Weights {
            bound_best_mem,
            last_max_mem,
            last_cur_mem,
            cur_mem,
            num_measure_nodes,
            num_total_nodes,
        })
    }

    #[pyo3(text_signature = "(self, bound_best_mem, last_max_mem, last_cur_mem, \
                             cur_mem, num_measure_nodes, num_total_nodes)")]
    fn __init__(
        &self,
        _bound_best_mem: f64,
        _last_max_mem: f64,
        _last_cur_mem: f64,
        _cur_mem: f64,
        _num_measure_nodes: f64,
        _num_total_nodes: f64,
    ) {
    }
}

#[pyo3::pymethods]
impl Shifts {
    #[new]
    fn __new__(
        upper_mem: f64,
        cur_mem: f64,
        time: f64,
        num_measure_nodes: f64,
        num_total_nodes: f64,
    ) -> Self {
        Self(probabilistic::Shifts {
            upper_mem,
            cur_mem,
            time,
            num_measure_nodes,
            num_total_nodes,
        })
    }

    #[pyo3(text_signature = "(self, upper_mem, cur_mem, time, num_measure_nodes, \
                             num_total_nodes)")]
    fn __init__(
        &self,
        _upper_mem: f64,
        _cur_mem: f64,
        _time: f64,
        _num_measure_nodes: f64,
        _num_total_nodes: f64,
    ) {
    }
}

#[pyo3::pymethods]
impl AcceptFunc {
    #[new]
    #[pyo3(signature = (
    kind="BuiltinBasic".to_string(),
    parametrized_basic_parameters=None,
    custom_func=None,
    ))]
    fn __new__(
        kind: String,
        parametrized_basic_parameters: Option<(Weights, Shifts)>,
        custom_func: Option<PyObject>,
    ) -> PyResult<Self> {
        Ok(Self(match kind.as_str() {
            "BuiltinBasic" => AcceptFuncBase::BuiltinBasic,
            "ParametrizedBasic" => {
                let parameters = match parametrized_basic_parameters {
                    Some(p) => p.clone(),
                    None => {
                        return Err(PyValueError::new_err(
                            "kind is ParametrizedBasic but \
                             parametrized_basic_parameters is None",
                        ));
                    },
                };
                AcceptFuncBase::ParametrizedBasic {
                    weights: parameters.0,
                    shifts: parameters.1,
                }
            },
            "Custom" => AcceptFuncBase::Custom(match custom_func {
                Some(f) => f.clone(),
                None => {
                    return Err(PyValueError::new_err(
                        "kind is Custom but custom_func is None",
                    ));
                },
            }),
            _ => return Err(PyValueError::new_err(format!("invalid kind {kind}"))),
        }))
    }

    /// Create a new AcceptFunc.
    ///
    /// Args:
    ///     kind (String): The kind of AcceptFunc to create; possible values are:
    ///         "BuiltinBasic", "ParametrizedBasic", "Custom".
    ///     parametrized_basic_parameters (tuple[Weights, Shifts]): The parameters for the
    ///         ParametrizedBasic AcceptFunc (if kind = "ParametrizedBasic").
    ///     custom_func (callable): The custom AcceptFunc (if kind = "Custom").
    #[pyo3(text_signature = "(self, kind='BuiltinBasic', \
                             parametrized_basic_parameters=None, custom_func=None)")]
    fn __init__(
        &self,
        _kind: String,
        _parametrized_basic_parameters: Option<(Weights, Shifts)>,
        _custom_func: Option<PyObject>,
    ) {
    }
}

impl AcceptFunc {
    pub(crate) fn to_real(&self) -> probabilistic::AcceptFunc {
        match self.0.clone() {
            AcceptFuncBase::BuiltinBasic => probabilistic::AcceptFunc::BuiltinBasic,
            AcceptFuncBase::ParametrizedBasic { weights, shifts } => {
                probabilistic::AcceptFunc::ParametrizedBasic {
                    weights: weights.0,
                    shifts: shifts.0,
                }
            },
            AcceptFuncBase::Custom(func) => probabilistic::AcceptFunc::Custom(Box::new(
                move |bound_best_mem,
                      last_max_mem: f64,
                      last_cur_mem: f64,
                      cur_mem: f64,
                      num_remaining_nodes: f64,
                      num_total_nodes: f64|
                      -> f64 {
                    Python::with_gil(|py| -> PyResult<f64> {
                        func.call(
                            py,
                            (
                                bound_best_mem,
                                last_max_mem,
                                last_cur_mem,
                                cur_mem,
                                num_remaining_nodes,
                                num_total_nodes,
                            ),
                            None,
                        )?
                        .extract(py)
                    }).expect("custom AcceptFunc failed")
                },
            )),
        }
    }
}

pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "probabilistic", parent_module.path.clone())?;
    module.add_class::<AcceptFunc>()?;
    module.add_class::<Weights>()?;
    module.add_class::<Shifts>()?;
    parent_module.add_submodule(py, module)?;
    Ok(())
}
