from matplotlib import pyplot as plt
import json

import utils

con = "constant"
lin = "reziprocal_linear"
root = "reziprocal_square_root"

EdgeDensityType = root
CorrectionDensityType = root

labels = [
    r"$S_{tt}$",
    r"$S_{sa}$",
    r"$S_{te}$",
    r"$S_{se}$",
]

xlabel = r"number of nodes $\abs{V}$"
ytlabel = r"time cost $\mathrm{tc}$"
yslabel = r"space cost $\mathrm{sc}$"


def node():
    # main()
    appendix()


def main():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.6))
    gs = fig.add_gridspec(1, 2)
    acs = []
    for i in range(2):
        acs.append(fig.add_subplot(gs[0, i]))
    # gs.update(hspace=0.02, wspace=0.21)
    gs.update(wspace=0.21)

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [colors[4], colors[3], colors[0], colors[2]]
    linestyles = ["solid", "dotted", "dashed", "dashdot"]

    # acs[0].plot([1, 2], [1, 2], color="white", label="$p_E$\hphantom{x}$p_C$")

    parameters = utils.get_parameters("node")

    para = parameters[0]
    data = get_data(para)["results"]

    max_x = 0

    for i in range(4):
        label = labels[i]
        color = colors[i]
        linestyle = linestyles[i]
        time = data[i * 4]
        space = data[i * 4 + 2]
        length = len(time)
        max_x = max(max_x, length)
        for j, dat in enumerate([time, space]):
            acs[j].plot(
                range(2, length + 2),
                dat,
                label=label,
                color=color,
                linestyle=linestyle,
            )

    for ac in acs:
        ac.set_xlim(2, max_x)
        ac.set_xlabel(xlabel)

    acs[0].set_ylabel(ytlabel)
    acs[1].set_ylabel(yslabel)

    utils.subplotlabel(acs[0], "a")
    utils.subplotlabel(acs[1], "b")

    handles, leg_labels = acs[0].get_legend_handles_labels()
    acs[0].legend(handles, leg_labels, loc="upper left", labelspacing=0.25)

    plt.subplots_adjust(top=0.95, bottom=0.10, left=0.07, right=0.97)
    plt.savefig(f"output/nodes_main.pdf")


def appendix():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.6))
    gs = fig.add_gridspec(1, 2)
    acs = []
    for i in range(2):
        acs.append(fig.add_subplot(gs[0, i]))
    # gs.update(hspace=0.02, wspace=0.21)
    gs.update(wspace=0.28)

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [colors[4], colors[3], colors[0], colors[2]]
    linestyles = ["solid", "dotted", "dashed", "dashdot"]

    parameters = utils.get_parameters("node")

    para = parameters[0]
    data = get_data(para)["results"]

    max_x = 0

    for i in range(4):
        label = labels[i]
        color = colors[i]
        linestyle = linestyles[i]
        time = data[i * 4]
        time_deviation = data[i * 4 + 1]
        space = data[i * 4 + 2]
        space_deviation = data[i * 4 + 3]
        length = len(time)
        max_x = max(max_x, length)
        for j, (mean, deviation) in enumerate(
            zip([time, space], [time_deviation, space_deviation])
        ):
            acs[j].errorbar(
                range(2, length + 2),
                mean,
                deviation,
                elinewidth=0.8,
                capsize=2,
                capthick=0.8,
                label=label,
                color=color,
                linestyle=linestyle,
            )

    for ac in acs:
        ac.set_xlim(2, max_x)
        ac.set_xlabel(xlabel)

    acs[0].set_ylabel(ytlabel)
    acs[1].set_ylabel(yslabel)

    utils.subplotlabel(acs[0], "a")
    utils.subplotlabel(acs[1], "b")

    handles, leg_labels = acs[0].get_legend_handles_labels()
    handles = [h[0] for h in handles]
    acs[0].legend(handles, leg_labels, loc="upper left", labelspacing=0.25)

    plt.subplots_adjust(top=0.95, bottom=0.10, left=0.07, right=0.97)
    plt.savefig(f"output/nodes_appendix.pdf")


def get_data(parameter: tuple[float, float]):
    file = (
        f"output/node-{EdgeDensityType}:{parameter[0]}_"
        f"{CorrectionDensityType}:{parameter[1]}.json"
    )
    with open(file, "r") as f:
        data = json.load(f)
    return data
