# Python wrapper around  mbqc_scheduling crate

This package provides an interface to the the [mbqc_scheduling crate]. It is usually used
together with the [Pauli tracker package].

## Documentation

[To the docs]

Since this here is mainly a wrapper, we did not document everything extensively, but
rather refer to the documentation of the underlying Rust crates, for now.

### Examples

See [examples/simple.py] for a very basic example. [examples/compile_flow.py] is a
specific example describing how the Pauli tracking and the scheduling can be used when
combining/stitching graphs that, for example, represent parts of a larger circuit (it's a
specific example for the project for which this library was initially developed).

## Installation

You can install the package from PyPI, e.g., with
```bash
pip install mbqc-scheduling
```
The package contains pre-built wheels for manylinux\_2\_28\_x86\_64 (works on most Linux
distribuitions), latest Windows and latest MacOS (latest with respect to when the package
was built) for Python 3.8 to 3.12. Additionally, there is an manylinux\_2\_28\_x86\_64
abi3 wheel for Python >= 3.8. You can also build the package from source, e.g., force it
during a pip install with `pip install --no-binary mbqc-scheduling mbqc-scheduling`,
however, note that this requires Python >= 3.8 and a Rust toolchain >= 1.65.

At the moment, you may also find a more up-to-date wheel in the artifacts of the latest
"pypackage" github actions workflow; this is unstable though.

### Manually Building

The package has to be build with [maturin]. The `make package` commands builds it through
a docker container such that it is compatible with manylinux\_2\_28\_x86\_64 for Python >=
3.8. With `make update_docs` the documentation can be build. The output of both make
commands is in the `dist` directory.

## Versioning

The Python package follows SemVer, however, the underlying Rust crate is unstable.

[examples/simple.py]: https://github.com/taeruh/mbqc_scheduling/blob/main/pauli_tracker/python_lib/examples/simple.py
[examples/compile_flow.py]: https://github.com/taeruh/mbqc_scheduling/blob/main/pauli_tracker/python_lib/examples/compile_flow.py
[manylinux]: https://github.com/pypa/manylinux
[maturin]: https://github.com/PyO3/maturin
[mbqc_scheduling crate]: https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling
[pauli_tracker crate's documentation]: https://docs.rs/pauli_tracker/latest/pauli_tracker/
[Pauli tracker package]: https://github.com/taeruh/pauli_tracker/tree/main/python_lib#readme
[To the docs]: https://taeruh.github.io/mbqc_scheduling/
[#1444]: https://github.com/PyO3/pyo3/issues/1444
