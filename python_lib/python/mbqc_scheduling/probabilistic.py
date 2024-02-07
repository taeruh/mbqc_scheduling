"""
Probabilistic accept functions (a little bit like a Markov chain), for the search
algorithm.

This module is very unstable at the moment and probably only useful if you know what
you are doing. When it is more stable, it will be better documented.

The content is aligned with the corresponding module in the `mbqc_scheduling crate
<https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling>`_; *for now, please
check out the documentation there for more details*.
"""

from mbqc_scheduling._lib.probabilistic import AcceptFunc, HeavysideParameters
