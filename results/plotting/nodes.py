from matplotlib import pyplot as plt
import json

import utils

con = "constant"
lin = "reziprocal_linear"
root = "reziprocal_square_root"

EdgeDensityType = root
CorrectionDensityType = root


def node():
    fig = plt.figure()
    gs = fig.add_gridspec(1, 2)
    acs = []
    for i in range(2):
        acs.append(fig.add_subplot(gs[0, i]))
    # gs.update(hspace=0.02, wspace=0.21)
    gs.update(wspace=0.21)

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"][0:4]
    labels = ["time trivial", "space search", "full search time", "full search space"]

    # acs[0].plot([1, 2], [1, 2], color="white", label="$p_E$\hphantom{x}$p_C$")

    parameters = utils.get_parameters("nodes")

    para = parameters[0]
    data = get_data(para)["results"]

    max_x = 0

    for i in range(4):
        color = colors[i]
        label = labels[i]
        time = data[i * 4]
        space = data[i * 4 + 2]
        length = len(time)
        max_x = max(max_x, length)
        for j, dat in enumerate([time, space]):
            acs[j].plot(
                range(2, length + 1),
                dat,
                label=label,
                color=color,
                # linestyle=linestyle,
            )

    for ac in acs:
        ac.set_xlim(1, max_x)
        ac.set_xlabel("num nodes")

    acs[0].set_ylabel("time")
    acs[1].set_ylabel("space")

    # time_up_lim = max(acs[0].get_ylim()[1], acs[1].get_ylim()[1])
    # space_up_lim = max(acs[2].get_ylim()[1], acs[3].get_ylim()[1])
    # # ticks = [i * 10 for i in range(1, 6)]
    # for i in range(4):
    #     acs[i].set_xlim(1, max_x)
    #     # acs[j].set_yticks(ticks)
    #     # acs[j].set_yticks(ticks)
    #     acs[i].tick_params(axis="y", which="both", right=True)
    #     if i > 1:
    #         acs[i].set_xlabel("num nodes")
    #         acs[i].set_ylim(1, space_up_lim)
    #     else:
    #         acs[i].set_xticklabels([])
    #         acs[i].set_ylim(1, time_up_lim)
    #     if i % 2 != 0:
    #         acs[i].set_yticklabels([])
    #     if i == 0:
    #         acs[i].set_ylabel("time")
    #         acs[i].set_title("time optimal")
    #     if i == 1:
    #         acs[i].set_title("space optimal (approx)")
    #     if i == 2:
    #         acs[i].set_ylabel("space")

    handles, labels = acs[0].get_legend_handles_labels()
    acs[0].legend(handles, labels, loc="upper left", labelspacing=0.25)

    plt.subplots_adjust(top=0.95, bottom=0.06, left=0.07, right=0.97)
    plt.savefig(f"output/nodes.pdf")


def get_data(parameter: tuple[float, float]):
    file = (
        f"output/nodes-{EdgeDensityType}:{parameter[0]}_"
        f"{CorrectionDensityType}:{parameter[1]}.json"
    )
    with open(file, "r") as f:
        data = json.load(f)
    return data
