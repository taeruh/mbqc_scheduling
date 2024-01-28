#!/usr/bin/env python


from pauli_tracker.frames.map import Frames
from pauli_tracker import scheduling
from pauli_tracker.scheduling import SpacialGraph
from pauli_tracker.scheduling.probabilistic import AcceptFunc, HeavysideParameters


def main():
    tracker, graph, frame_flags = data()
    time_order = tracker.get_order(frame_flags)
    paths = scheduling.run(
        graph,
        time_order,
        do_search=True,
        nthreads=10,
        # probabilistic=AcceptFunc(),
        # probabilistic=AcceptFunc(kind="Custom", custom_func=custom),
        # probabilistic=AcceptFunc(
        #     kind="ParametrizedHeavyside",
        #     heavyside_parameters=HeavysideParameters(0, 2, 1, 1, 3, 1),
        # ),
    ).into_py_paths()
    for path in paths:
        print(f"time: {path.time}, space: {path.space}, steps: {path.steps}")


def custom(
    bound_best_mem,
    minimal_mem,
    last_max_mem,
    last_cur_mem,
    cur_mem,
    num_remaining_nodes,
    num_total_nodes,
):
    return 0.5


def data():
    graph = SpacialGraph.deserialize("../../../test_files/fourier_4o_spacial.json")
    tracker = Frames.deserialize("../../../test_files/fourier_4o_frames.json")
    frame_flags = [3, 4, 5, 6, 7, 8, 2, 10, 11, 12, 1, 14]
    return tracker, graph, frame_flags


if __name__ == "__main__":
    main()
