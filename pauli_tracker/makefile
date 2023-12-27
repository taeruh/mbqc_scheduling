# merge helper to quickly remove everything that we do not need; we keep:
# - python_lib/
# - pauli_tracker/tests/roundtrips/tracking.rs
# the latter one is for the roundtrip test in ../mbqc_scheduling/

SHELL=/bin/bash # for path/{a,b} expansion

clean:
	rm -rf .github/ c_lib/ CONTRIBUTING.md docs/ LICENSE-APACHE LICENSE-MIT README.md rustfmt.toml TODO.md xtask/ python_lib/xtask
	rm -rf pauli_tracker/{archive/,src/,xdocs/}
	rm -rf pauli_tracker/tests/roundtrips/mod.rs
