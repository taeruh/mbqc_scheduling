#!/usr/bin/env python

# pyright: reportUnusedVariable=warning


from pauli_tracker.frames.map import Frames
from pauli_tracker.scheduling import SpacialGraph
from pauli_tracker import scheduling
from pauli_tracker.probabilistic import AcceptFunc, Weights, Shifts

import time


def standard_accept_func(
    bound_best_mem,
    _last_max_mem,
    _last_cur_mem,
    cur_mem,
    num_remaining_nodes,
    num_total_nodes,
):
    return (
        (bound_best_mem + 1.0)
        / (cur_mem + 1.0)
        * (
            1e-3
            + 8.5e-2
            * (num_total_nodes + 1.0)
            / (num_total_nodes - num_remaining_nodes + 1.0)
        )
    )


def main():
    frames_map = [3, 4, 5, 6, 7, 8, 2, 10, 11, 12, 1, 14]
    tracker = Frames.deserialize("../../../test_files/fourier_oooo_frames.json")
    spacial_graph = SpacialGraph.deserialize(
        "../../../test_files/fourier_oooo_spacial.json"
    )
    dep_graph = tracker.create_dependency_graph(frames_map)

    accept_func = None

    # accept_func = AcceptFunc(kind="BuiltinBasic")

    # kind = AcceptFuncKind.CreateFunc
    # weights = Weights(1.0, 0.0, 1.0, 1.0, 1.3e-1)
    # shifts = Shifts(1.0, 1.0, 1e-3, 1.0, 1.3e-1)
    # accept_func = AcceptFunc(kind, CreateFuncParameters(weights, shifts))

    # accept_func = AcceptFunc(kind="Custom", custom_func=standard_accept_func)

    start = time.time()
    print(scheduling.run(spacial_graph, dep_graph, True, 1, accept_func, None))
    print(time.time() - start)


if __name__ == "__main__":
    main()
