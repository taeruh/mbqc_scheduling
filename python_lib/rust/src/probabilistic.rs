use lib::probabilistic;
use pyo3::{
    PyObject, PyResult, Python, exceptions::PyValueError, types::PyModuleMethods,
};

use crate::Module;

#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct HeavysideParameters(probabilistic::HeavysideParameters);

#[pyo3::pymethods]
impl HeavysideParameters {
    #[new]
    fn __new__(
        cutoff: f64,
        lin_num_total_nodes_exp: i32,
        exp_num_total_nodes_exp: i32,
        exp_num_remaining_nodes_exp: i32,
        exp_diff_exp: i32,
        exp_num_measured_nodes_exp: i32,
    ) -> Self {
        Self(probabilistic::HeavysideParameters {
            cutoff,
            lin_num_total_nodes_exp,
            exp_num_total_nodes_exp,
            exp_num_remaining_nodes_exp,
            exp_diff_exp,
            exp_num_measured_nodes_exp,
        })
    }

    #[pyo3(text_signature = "(self, cutoff, lin_num_total_nodes_exp, \
                             exp_num_total_nodes_exp, exp_num_remaining_nodes_exp, \
                             exp_diff_exp, exp_num_measured_nodes_exp)")]
    fn __init__(
        &self,
        _cutoff: f64,
        _lin_num_total_nodes_exp: i32,
        _exp_num_total_nodes_exp: i32,
        _exp_num_remaining_nodes_exp: i32,
        _exp_diff_exp: i32,
        _exp_num_measured_nodes_exp: i32,
    ) {
    }
}

// since the original probabilistic::AcceptFunc cannot implement Clone
#[derive(Clone)]
pub enum AcceptFuncBase {
    BuiltinHeavyside,
    ParametrizedHeavyside {
        param: probabilistic::HeavysideParameters,
    },
    Custom(PyObject),
}

#[pyo3::pyclass(subclass)]
/// Compare the corresponding documentation in the `mbqc_scheduling crate
/// <https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_.
#[derive(Clone)]
pub struct AcceptFunc(pub AcceptFuncBase);

#[pyo3::pymethods]
impl AcceptFunc {
    #[new]
    #[pyo3(signature = (
    kind="BuiltinHeavyside".to_string(),
    heavyside_parameters=None,
    custom_func=None,
    ))]
    fn __new__(
        kind: String,
        heavyside_parameters: Option<HeavysideParameters>,
        custom_func: Option<PyObject>,
    ) -> PyResult<Self> {
        Ok(Self(match kind.as_str() {
            "BuiltinHeavyside" => AcceptFuncBase::BuiltinHeavyside,
            "ParametrizedHeavyside" => {
                let param = match heavyside_parameters {
                    Some(p) => p,
                    None => {
                        return Err(PyValueError::new_err(
                            "kind is ParametrizedHeavyside but heavyside_parameters is \
                             None",
                        ));
                    },
                };
                AcceptFuncBase::ParametrizedHeavyside { param: param.0 }
            },
            "Custom" => AcceptFuncBase::Custom(match custom_func {
                Some(f) => f,
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
    ///         "BuiltinHeavyside", "ParamBuiltinHeavyside", "Custom".
    ///     heavyside_parameters (Optional[HeavysideParameters]): The
    ///         parameters for the ParametrizedHeavyside AcceptFunc (if kind =
    ///         "ParametrizedBasic").
    ///     custom_func (callable): The custom AcceptFunc (if kind = "Custom").
    #[pyo3(text_signature = "(self, kind='BuiltinLinearSpace', \
                             heavyside_parameters=None, custom_func=None)")]
    fn __init__(
        &self,
        _kind: String,
        _heavyside_parameters: Option<HeavysideParameters>,
        _custom_func: Option<PyObject>,
    ) {
    }
}

impl AcceptFunc {
    pub(crate) fn to_real(&self) -> probabilistic::AcceptFunc {
        match self.0.clone() {
            AcceptFuncBase::BuiltinHeavyside => {
                probabilistic::AcceptFunc::BuiltinHeavyside
            },
            AcceptFuncBase::ParametrizedHeavyside { param } => {
                probabilistic::AcceptFunc::ParametrizedHeavyside { param }
            },
            AcceptFuncBase::Custom(func) => probabilistic::AcceptFunc::Custom(Box::new(
                move |bound_best_mem,
                      minimal_mem,
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
                                minimal_mem,
                                last_max_mem,
                                last_cur_mem,
                                cur_mem,
                                num_remaining_nodes,
                                num_total_nodes,
                            ),
                            None,
                        )?
                        .extract(py)
                    })
                    .expect("custom AcceptFunc failed")
                },
            )),
        }
    }
}

pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "probabilistic", parent_module.path.clone())?;
    module.pymodule.add_class::<AcceptFunc>()?;
    module.pymodule.add_class::<HeavysideParameters>()?;
    parent_module.add_submodule(py, module)?;
    Ok(())
}
