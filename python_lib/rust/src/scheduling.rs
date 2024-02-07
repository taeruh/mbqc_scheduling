pub fn add_module(py: Python<'_>, parent_module: &Module) -> PyResult<()> {
    let module = Module::new(py, "scheduling", parent_module.path.clone())?;
    parent_module.add_submodule(py, module)?;
    Ok(())
}
