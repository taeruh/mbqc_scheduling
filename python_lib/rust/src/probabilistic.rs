use lib::probabilistic;
use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyValueError, types::PyModuleMethods,
};

use crate::Module;

// #[pyo3(text_signature = "(self, cutoff, lin_num_total_nodes_exp, \
//                          exp_num_total_nodes_exp, exp_num_remaining_nodes_exp, \
//                          exp_diff_exp, exp_num_measured_nodes_exp)")]

#[pyo3::pyclass(subclass,from_py_object)]
/// **Constructor:**
/// Args:
///     cutoff (float)
///     lin_num_total_nodes_exp (int)
///     exp_num_total_nodes_exp (int)
///     exp_num_remaining_nodes_exp (int)
///     exp_diff_exp (int)
///     exp_num_measured_nodes_exp (int)
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
}

// since the original probabilistic::AcceptFunc cannot implement Clone
#[derive(Clone)]
pub enum AcceptFuncBase {
    BuiltinHeavyside,
    ParametrizedHeavyside {
        param: probabilistic::HeavysideParameters,
    },
    Custom(Py<PyAny>),
}

#[pyo3::pyclass(subclass,from_py_object)]
/// Compare the corresponding documentation in the `mbqc_scheduling crate
/// <https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_.
///
/// **Constructor:**
///
/// Args:
///     kind (String): The kind of AcceptFunc to create; possible values are:
///         "BuiltinHeavyside", "ParamBuiltinHeavyside", "Custom".
///     heavyside_parameters (Optional[HeavysideParameters]): The
///         parameters for the ParametrizedHeavyside AcceptFunc (if kind =
///         "ParametrizedBasic").
///     custom_func (callable): The custom AcceptFunc (if kind = "Custom").
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
        custom_func: Option<Py<PyAny>>,
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
                    Python::attach(|py| -> PyResult<f64> {
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
