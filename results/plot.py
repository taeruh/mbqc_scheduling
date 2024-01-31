#!/usr/bin/env python

from matplotlib import pyplot as plt
import json


def main():
    parameters = get_parameters()

    fig = plt.figure()
    gs = fig.add_gridspec(2, 2)
    axes = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        axes.append(fig.add_subplot(gs[i, j]))
    map = [0, 2, 1, 3]

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"][0:3]
    linestyles = ["solid", "dashed", "dotted"]

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

        for j in range(4):
            axes[j].plot(
                x, y[j], label=f"{para[0]} {para[1]}", color=color, linestyle=linestyle
            )

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
    with open(f"output/{parameter[0]}_{parameter[1]}.json", "r") as f:
        data = json.load(f)
    return data


if __name__ == "__main__":
    main()
