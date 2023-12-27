#!/usr/bin/env python


from pauli_tracker.frames.map import Frames
from pauli_tracker.scheduling import SpacialGraph
from pauli_tracker import scheduling


def main():
    frames_map = [3, 4, 5, 6, 7, 8, 2, 10, 11, 12, 1, 14]
    tracker = Frames.deserialize("../../../test_files/fourier_oooo_frames.json")
    spacial_graph = SpacialGraph.deserialize(
        "../../../test_files/fourier_oooo_spacial.json"
    )
    dep_graph = tracker.create_dependency_graph(frames_map)
    print(scheduling.run(spacial_graph, dep_graph, False, 1, None))


if __name__ == "__main__":
    main()
