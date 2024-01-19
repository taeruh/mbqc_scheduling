#!/usr/bin/env python

# this is an example of how the pauli tracking can be integrated into a compile flow where
# we stitch widgets together

# the following example can basically be one-to-one translated to julia by installing the
# python package via (using the PythonCall package)
# ```julia
# using PythonCall
# using CondaPkg
# CondaPkg.add_pip(
#     "pauli_tracker";
#     version="@ ./path/to/the/pauli_tracker/whl-file"
# )
# ```
# and replacing `import this` and `from this import that` to `this = pyimport("this")` and
# `that = pyimport("this").that`, repectively.
#
#
# If there are problems with installing the package (or if you see some other weird stuff,
# like installing stim although it is nowhere specified ...), removing the local
# `.CondaPkg` directory and doing something like the following helps (fd is a find command
# ...):
# ```
# # ONLY DO THIS IF YOU KNOW WHAT YOU ARE DOING, I take no responsibility if you
# # accidentally delete something important
# cd ~/.julia
# for d in (fd PythonCall); rm -rf $d; end
# for d in (fd CondaPkg); rm -rf $d; end
# ```

# please have a look at the simple.py example first

from pauli_tracker.frames.map import Frames
from pauli_tracker import scheduling
from pauli_tracker.scheduling import SpacialGraph
from pauli_tracker.pauli import PauliStack, PauliTuple


def main():
    # lets to compile the first widget, widget_a

    # do ruby_slippers or jabalizer with integrated pauli tracking
    tracker, frame_flags, graph, local_clifford_corrections = compile_widget_a()
    # get the graph that describes the partial (time) order of the measurements
    time_order = tracker.get_order(frame_flags)
    # get the time optimal initialization-measurement pattern; at the moment we don't care
    # about space optimality, but when we care, the `run` function also has some other
    # parameters to get that (cf. its docs)
    path = scheduling.run(SpacialGraph(graph), time_order).into_py_paths()[0]
    # interesting for resource estimation:
    # print(f"time: {path.time}, space: {path.space}, steps: {path.steps}")

    # now we have all the information that is needed to execute the pattern for widget_a;
    # you probably want to serialize it (note that most of the types in pauli_tracker have
    # a serialize and deserialize method where you can choose between different formats);
    # doing this in this example is unnecessary, so we just "store" it; note that we store
    # the frames transpose (in Frames, it is major-qubit-minor-frame, but when actually
    # running it, we probably want major-frame-minor-qubit, because each time a qubit is
    # measured, we want to access the frame)
    storage_a = Storage(
        tracker.stacked_transpose_reverted(len(graph)),
        frame_flags,
        graph,
        path,
        local_clifford_corrections,
    )

    # now let's do widget_b

    # widget_a is going to induce corrections into widget_b; there are two ways to deal
    # with that: 1) we could just use the above tracker from widget a for widget, this
    # means we have in the end just one global tracker. However, we don't want to do that
    # since we want the frames separated (for multiple reasons). Therefore we choose the
    # other option: 2) use a freshly initialized tracker and additionally have a buffer
    # tracker to capture potential corrections which based on the corrections that are on
    # the output qubits of widget_a
    buffer = buffer_tracker(3)  # 3, since we have 3 output (input) in a (b)

    # now as before:
    tracker, frame_flags, graph, local_clifford_corrections = compile_widget_b(buffer)
    path = scheduling.run(
        SpacialGraph(graph), tracker.get_order(frame_flags)
    ).into_py_paths()[0]
    storage_b = Storage(
        tracker.stacked_transpose_reverted(len(graph)),
        frame_flags,
        graph,
        path,
        local_clifford_corrections,
        buffer.stacked_transpose_reverted(len(graph)),
    )

    # note that while `tracker` scales quadratically with the total number of nodes in the
    # widget's graph, `buffer` scales linearly in the total number of nodes and the number
    # of input/output nodes

    # similar do widget_c, widget_d, ...

    # that was basically the "compilation" part; now we are going to "run" it
    #
    # the following is just pseudo code, I don't actually want to include the code to run;
    # I might have forgotten something here (this here focuses only on the pauli
    # corrections); see, for example, the check.py script in the jabalizer branch for how
    # to do it with qiskit (note that when doing things with qiskit, we are not actually
    # transposing the frames, because we can not really dynamically do a quantum operation
    # depending on a measurement result (which is basically needed for mbqc), but instead
    # do this hacky way with multiple corrections); there's of course also some logic for
    # the stitching of the graphs needed

    # starting with widget_a

    # this frame will be used to capture the Pauli corrections we need to perform (or
    # adjust the measurements)
    frame = PauliStack.zeros(len(storage_a.graph))

    for step in storage_a.path.steps:
        for node in step:
            # pseudo: init qubit node and it's neighbors graph[node] and create the edges
            # pseudo: perform local_clifford_corrections[node]
            pauli_correction = frame.get(node).into_py_tuple()
            if pauli_correction[0]:
                pass  # pseudo: perform Pauli Z correction on node
            elif pauli_correction[1]:
                pass  # pseudo: perform Pauli X correction on node
            # pseudo: measure node
            measurement_outcome = True  # random ...
            if measurement_outcome and node in storage_a.frame_flags:
                # an additional hashmap would probably be senseful for the frame_flags...
                idx = storage_a.frame_flags.index(node)
                storage_a.frames.get_and_add_to_stack(idx, frame)

    # that was widget_a (if I didn't forget something); now widget_b

    # first we do the stitching
    # pseudo: do the stitching similar as to what is done above, i.e., consider the
    #   local_clifford_corrections and the according storage_a.frames

    # imagine these are the final corrections that come from the output qubits of widget_a
    output_corrections = [
        PauliTuple(True, False),  # Z
        PauliTuple(False, True),  # X
        PauliTuple(True, True),  # Y
    ]  # random ...

    frame = PauliStack.zeros(len(storage_b.graph))

    # account for the corrections from widget_a
    for i, correction in enumerate(output_corrections):
        correction = correction.into_py_tuple()
        if correction[0]:
            storage_b.buffer.get_and_add_to_stack(i, frame)  # pyright: ignore
        if correction[1]:
            storage_b.buffer.get_and_add_to_stack(i + 1, frame)  # pyright: ignore

    # now do the same loop as above

    # widget_c, widget_d, ...


def compile_widget_a() -> tuple[Frames, list[int], list[list[int]], list]:
    # imagine this is an actually widget, then you want to do the tracking according to
    # how the circuit is transformed into a graph; I'm just doing some random tracking
    # here with a random graph,
    tracker = Frames(3)
    frame_flags = []
    tracker.new_qubit(3)
    tracker.cz(0, 3)
    tracker.track_x(3)
    frame_flags.append(0)
    tracker.cz(2, 3)
    tracker.new_qubit(4)
    tracker.cz(2, 4)
    tracker.track_z(4)
    frame_flags.append(2)
    tracker.h(4)
    tracker.cy(4, 1)
    # sparse graph representation; index is the node
    graph = [[1, 2], [0, 2, 3], [0, 1], [1, 4], [3]]
    local_clifford_corrections = []
    return tracker, frame_flags, graph, local_clifford_corrections


def compile_widget_b(buffer: Frames) -> tuple[Frames, list[int], list[list[int]], list]:
    # again some random stuff; every time we do a gate or add a qubit on the "real"
    # tracker, we do the same on the buffer tracker
    tracker = Frames(3)
    frame_flags = []
    tracker.h(1)
    buffer.h(1)
    tracker.new_qubit(3)
    buffer.new_qubit(3)
    tracker.cz(2, 3)
    buffer.cz(2, 3)
    tracker.track_x(3)
    # buffer.track_x(3)  # NO: no additional frames on the buffer tracker!!!
    frame_flags.append(2)
    tracker.iswap(3, 1)
    buffer.iswap(3, 1)
    tracker.new_qubit(4)
    buffer.new_qubit(4)
    tracker.cz(1, 4)
    buffer.cz(1, 4)
    tracker.track_y(4)
    frame_flags.append(1)
    # to lazy to think of another graph
    graph = [[1, 2], [0, 2, 3], [0, 1], [1, 4], [3]]
    local_clifford_corrections = []
    return tracker, frame_flags, graph, local_clifford_corrections


# I'm assuming here that the qubits are always continuously labeled starting from 0 for
# each widget
def buffer_tracker(len: int) -> Frames:
    tracker = Frames(len)
    for i in range(len):
        tracker.track_z(i)
        tracker.track_x(i)
    return tracker


class Storage:
    def __init__(
        self, frames, frame_flags, graph, path, local_clifford_corrections, buffer=None
    ):
        self.frames = frames
        self.frame_flags = frame_flags
        self.graph = graph
        self.path = path
        self.local_clifford_corrections = local_clifford_corrections
        self.buffer = buffer


if __name__ == "__main__":
    main()
