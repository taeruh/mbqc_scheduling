use std::ops::Deref;

use pyo3::{
    types::PyModule,
    PyResult,
    Python,
};

struct Module<'m> {
    pymodule: &'m PyModule,
    path: String,
}

impl<'m> Module<'m> {
    fn new(py: Python<'m>, name: &str, mut path: String) -> PyResult<Self> {
        path.push_str(format!(".{}", name).as_str());
        Ok(Self {
            pymodule: PyModule::new(py, name)?,
            path,
        })
    }

    fn add_submodule(&self, py: Python<'_>, submodule: Self) -> PyResult<()> {
        self.pymodule.add_submodule(submodule.pymodule)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item(submodule.path, submodule.pymodule)?;
        Ok(())
    }
}

impl Deref for Module<'_> {
    type Target = PyModule;
    fn deref(&self) -> &Self::Target {
        self.pymodule
    }
}

mod impl_helper;

mod frames;
mod live;
mod pauli;
mod scheduling;

#[pyo3::pymodule]
fn _lib(py: Python, module: &PyModule) -> PyResult<()> {
    let module = Module {
        pymodule: module,
        path: "pauli_tracker._lib".to_string(),
    };

    live::add_module(py, &module)?;
    frames::add_module(py, &module)?;
    pauli::add_module(py, &module)?;
    scheduling::add_module(py, &module)?;

    Ok(())
}
