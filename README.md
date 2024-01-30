# MBQC Scheduling

In this project we tackle the task of finding optimal initialization-measurement patterns
for MBQC, on an abstract level. The problem is the following: We have a *spacial graph*
that represents where the nodes represent qubits and the edges represent entanglement. The
nodes can be in three states: *uninitialized*, *initialized* and *measured*. The goal is
to consume the whole graph, i.e., to put all nodes into the *measured* states. However,
there are certain rules that have to be followed:
1. A node can only be measured if it is initialized.
2. A node can only be measured if all its neighbors are initialized.
3. Additionally to the graph, we have a *partial time ordering* with respect to measuring
   nodes, that is, some nodes have to be measured before others.

For example, a trivial solution would be to first initialized all nodes, then measure them
according to the partial time ordering. However, this is not space efficient, in the sense
that at one point, all nodes are initialized, which is  assumed to be costly. So to
clarify the goal, we want to find a pattern/path of measurement steps - each step defines
a number of nodes that are measured at the same time - such that the maximum number of
nodes that was initialized at any point in time is kept small, while the number of these
measurement steps is kept small as well. This is the *space-time* optimization problem we
want to solve. The output should be the a maximally time efficient pattern (keeping the
space as small as possible), and a maximally space efficient pattern (keeping the time as
as small as possible), and everything in between.

This problem is NP-complete, I think, so approximations are needed. Currently, we solve it
by doing doing a brute force search over all possible patterns (skipping patterns that are
obviously not optimal), and putting something like a probabilistic Markov chain on top of
it (it's not really a Markov chain, but kinda similar). The performance depends highly on
the graph structure and the partial time ordering, as well as the chosen
accept-probability function in the "Markov chain". In the worst case, the brute force
search itself scales between factorial and double-exponentially with respect to the number
of nodes in the graph.

## Usage

The wording of the implementation is oriented at MBQC and Pauli tracking - the latter
defines the partial time ordering - since this is the application we had in mind. In the
[mbqc_scheduling crate] is the implementation as a Rust library (however without any
guarantees regarding API stability). You probably want to use it through this [Python
package], which is an extension of the [Pauli tracker package] including an interface to
the [mbqc_scheduling crate].

[mbqc_scheduling crate]: https://github.com/taeruh/mbqc_scheduling/tree/main/mbqc_scheduling
[Python package]: https://github.com/taeruh/mbqc_scheduling/tree/main/pauli_tracker/python_lib#readme
[Pauli tracker package]: https://github.com/taeruh/pauli_tracker/tree/main/python_lib#readme

## License

MBQC Scheduling is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).
