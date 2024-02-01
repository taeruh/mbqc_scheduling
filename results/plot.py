#!/usr/bin/env python

from matplotlib import pyplot as plt
import json

con = "Constant"
lin = "ReziprocalLinear"

EdgeDensityType = lin
CorrectionDensityType = lin


def main():
    parameters = get_parameters()

    fig = plt.figure()
    gs = fig.add_gridspec(2, 2)
    axes = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        axes.append(fig.add_subplot(gs[i, j]))
    map = [0, 2, 1, 3]
    gs.update(hspace=0.0, wspace=0.0)

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"][0:3]
    linestyles = ["solid", "dashed", "dotted"]

    max_x = 0

    for i, para in enumerate(parameters):
        color = colors[i // 3]
        linestyle = linestyles[i % 3]

        data = get_data(para)["results"]
        x = []
        y = [[] for _ in range(4)]
        for dat in data:
            x.append(dat[0])
            for j in range(4):
                y[map[j]].append(dat[1][j][0])

        max_x = max(max_x, len(x))

        for j in range(4):
            axes[j].plot(
                x, y[j], label=f"{para[0]} {para[1]}", color=color, linestyle=linestyle
            )

    for j in range(4):
        axes[j].set_xlim(0, max_x)
        if j > 1:
            axes[j].set_xlabel("num nodes")
        else:
            axes[j].set_xticklabels([])
        if j % 2 != 0:
            axes[j].set_yticklabels([])
        if j == 0:
            axes[j].set_ylabel("time")
            axes[j].set_title("time optimal")
        if j == 1:
            axes[j].set_title("space optimal (approx)")
        if j == 2:
            axes[j].set_ylabel("space")

    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="upper right")
    plt.show()

    # y = []
    # x = []
    # for dat in data["results"]:
    #     x.append(dat[0])
    #     y.append(dat[1][0][0])
    # plt.plot(x, y)
    # plt.show()


def get_parameters():
    parameters = []
    with open("parameters.dat", "r") as f:
        for pair in f.read().splitlines():
            pair = pair.split(" ")
            parameters.append((float(pair[0]), float(pair[1])))
    return parameters


def get_data(parameter: tuple[float, float]):
    with open(
        (
            f"output/factor:{parameter[0]},typ:{EdgeDensityType}_"
            f"factor:{parameter[1]},typ:{CorrectionDensityType}.json"
        ),
        "r",
    ) as f:
        data = json.load(f)
    return data


if __name__ == "__main__":
    main()
