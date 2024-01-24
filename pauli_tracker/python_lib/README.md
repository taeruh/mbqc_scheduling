# Python wrapper around the pauli_tracker and mbqc_scheduling crates

This package is an extension of the [Pauli tracker package] including an interface to the
the [mbqc_scheduling crate]. If you already use the [Pauli tracker package], uninstall it.
This package here is a superset of it.

This weird extension is because of [#1444].

## Documentation

[To the docs](https://taeruh.github.io/mbqc_scheduling/). Since this here is mainly a
wrapper, we did not document everything extensively, but rather refer to the documentation
of the underlying Rust crates, especially when it is about Pauli tracking ([pauli_tracker
crate's documentation]).

## Examples

See [examples/simple.py] for a very basic example. [examples/compile_flow.py] is a
specific example describing how the Pauli tracking and the scheduling can be used when
combining/stitching graphs that, for example, represent parts of a larger circuit (it's a
specific example for the project for which this library was initially developed).

## Installation

Until we attach the built package to release tags, you can download it from the artifacts
of the Github actions that have "pypackage" as workflow, e.g., from the [latest build].
Just choose the right build for your OS and Python version (for Linux, the builds for the
different Python versions are all bundled in the "linux-wheels" artifact; they are all
build for manylinux\_2\_28\_x86\_64, cf [manylinux]; the artifact also contains an abi3
build for Python>=3.8). You may have to unzip the artifact. Then you can install the
package with `pip install <path-to-whl-file>`.

## Manually Building

The package has to be build with [maturin]. The `make package` commands builds it through
a docker container such that it is compatible with manylinux\_2\_28\_x86\_64 for Python >=
3.8. With `make update_docs` the documentation can be build. The output of both make
commands is in the `dist` directory.

## SemVer

The API of the underling Rust crate is not stable (but the Python package will follow
SemVer as seen as we release a first version).

[examples/simple.py]: https://github.com/taeruh/mbqc_scheduling/blob/main/pauli_tracker/python_lib/examples/simple.py
[examples/compile_flow.py]: https://github.com/taeruh/mbqc_scheduling/blob/main/pauli_tracker/python_lib/examples/compile_flow.py
[pauli_tracker crate's documentation]: https://docs.rs/pauli_tracker/latest/pauli_tracker/
[Pauli tracker package]: https://github.com/taeruh/pauli_tracker/tree/main/python_lib#readme
[latest build]: https://github.com/taeruh/mbqc_scheduling/actions/runs/7633528141
[manylinux]: https://github.com/pypa/manylinux
[maturin]: https://github.com/PyO3/maturin
[mbqc_scheduling crate]: https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling
[#1444]: https://github.com/PyO3/pyo3/issues/1444
