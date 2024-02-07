#!/usr/bin/env python

from pauli_tracker.frames.map import Frames
import mbqc_scheduling
from mbqc_scheduling import SpacialGraph, PartialOrderGraph


def main():
    # get our example data; tracker contains the tracked Pauli frames which will define
    # the time ordering; graph is the spacial graph defining the nodes/qubits and
    # edges/entanglements; frame_flags is a list that describes on which measurements the
    # frames depend, that is, the induced correction by frame[i] have are applied
    # depending on the outcome of the measurment of qubit frame_flags[i]
    tracker, graph, frame_flags = data()

    # calculate the partial time order graph
    time_order = tracker.get_order(frame_flags)
    # the following is not needed, but it avoids a warning and if we would call `run`
    # mutltiple times with this time_order, it avoids cloning the data structure
    time_order = PartialOrderGraph(time_order.take_into_py_graph())

    # get a time-optimal initialization-measurement pattern/path (cf. docs for other
    # things you can do with this function)
    path = mbqc_scheduling.run(graph, time_order).into_py_paths()[0]

    # `time` is the number of parallel measurements that is needed (it's just the length
    # of `steps`; `space` is the number of qubits required to execute the pattern (taking
    # into account, that when a qubit is measured, all its neighbors have to be
    # initialized); `steps` is the list of these parallel measurements
    print(f"time: {path.time}, space: {path.space}, steps: {path.steps}")


# A block box that returns the data for our simple example.
# In a real application, you get this data from tracking the Pauli corrections when
# building up an MBQC circuit.
def data():
    graph = SpacialGraph.deserialize("../../test_files/fourier_4o_spacial.json")
    tracker = Frames.deserialize("../../test_files/fourier_4o_frames.json")
    frame_flags = [3, 4, 5, 6, 7, 8, 2, 10, 11, 12, 1, 14]
    return tracker, graph, frame_flags


if __name__ == "__main__":
    main()
