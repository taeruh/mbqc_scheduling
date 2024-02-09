#!/usr/bin/env python

# ugly as ..., but I don't care

from matplotlib import pyplot as plt
import json

con = "constant"
lin = "reziprocal_linear"
root = "reziprocal_square_root"

task = "node"
# task = "density"

if task == "node":
    EdgeDensityType = root
    CorrectionDensityType = root
    size = 0  # doesn't matter at the moment
else:
    EdgeDensityType = con  # doesn't matter at the moment
    CorrectionDensityType = con  # doesn't matter at the moment
    size = 10


def main():
    paper_setup()
    if task == "node":
        node()
    else:
        density()


def density():
    fig = plt.figure()
    gs = fig.add_gridspec(2, 2)
    acs = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        acs.append(fig.add_subplot(gs[i, j]))
    map = [0, 2, 1, 3]
    gs.update(hspace=0.04, wspace=-0.4)

    data = [[] for _ in range(4)]

    parameters = get_parameters()
    for para in parameters:
        dat = get_data(para)["results"]
        for j in range(4):
            data[map[j]].append(dat[2 * j])

    images = []
    for i, dat in enumerate(data):
        images.append(acs[i].imshow(dat, origin="lower"))

    for i in range(4):
        acs[i].grid(False)
        if i > 1:
            acs[i].set_xlabel("correction density")
        else:
            acs[i].set_xticklabels([])
        if i % 2 == 0:
            acs[i].set_ylabel("edge density")
        else:
            acs[i].set_yticklabels([])
        if i == 0:
            acs[i].set_title("time optimal")
        if i == 1:
            acs[i].set_title("space optimal (approx)")
            acs[i].text(1.05, 0.5, "time cost", transform=acs[i].transAxes, rotation=45)
        if i == 3:
            acs[i].text(
                1.05, 0.5, "space cost", transform=acs[i].transAxes, rotation=45
            )

    fig.colorbar(images[0], ax=acs, orientation="horizontal", fraction=0.1)

    plt.savefig(f"output/{task}.pdf")


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

    parameters = get_parameters()

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
    plt.savefig(f"output/{task}.pdf")


def get_parameters():
    parameters = []
    with open(f"parameters/{task}.dat", "r") as f:
        for pair in f.read().splitlines():
            pair = pair.split(" ")
            parameters.append((float(pair[0]), float(pair[1])))
    return parameters


# def get_data(parameter: tuple[float, float]):
#     with open(f"output/{parameter[0]}_{parameter[1]}.json", "r") as f:
#         data = json.load(f)
#     return data


def get_data(parameter: tuple[float, float]):
    if task == "node":
        file = (
            f"output/{task}-{EdgeDensityType}:{parameter[0]}_"
            f"{CorrectionDensityType}:{parameter[1]}.json"
        )
    else:
        file = f"output/{task}-size:{size}_density:{int(parameter[1])}.json"
    with open(file, "r") as f:
        data = json.load(f)
    return data


def paper_setup():
    plt.style.use(["./plotting/ownstandard.mplstyle", "./plotting/ownlatex.mplstyle"])
    plt.rcParams.update(
        {
            # "figure.figsize": [*set_size()],
            "font.size": 10,
            "lines.linewidth": 1.5,
        }
    )


# get default with \the\textwidth
def set_size(width_in_pt=510.0, height_in_width=1.0, scale=1.0):
    width_in_in = width_in_pt * scale / 72.27
    return (width_in_in, width_in_in * height_in_width)


if __name__ == "__main__":
    main()
