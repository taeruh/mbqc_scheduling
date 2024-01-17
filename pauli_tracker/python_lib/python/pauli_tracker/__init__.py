"""
Wrapper around the essential functionality of the `pauli_tracker crate
<https://docs.rs/pauli_tracker/latest/pauli_tracker>`_, and the `mbqc_scheduling crate`_.

When exporting Rust code through the FFI, we loose the ability to be generic. Because of
that we can only support specific types in this wrapper. The submodule structure kinda
emulates these types. For example, the :obj:`Live <.live.map.Live>` in :mod:`.live.map`
corresponds to Rust's `Live`_\\<`Map`_\\<_>> type.

   
.. _mbqc_scheduling crate:
   https://github.com/taeruh/mbqc_scheduling/blob/main/mbqc_scheduling
.. _Live:
   https://docs.rs/pauli_tracker/latest/pauli_tracker/tracker/live/struct.Live.html
.. _Map:
   https://docs.rs/pauli_tracker/latest/pauli_tracker/collection/type.Map.html
"""
