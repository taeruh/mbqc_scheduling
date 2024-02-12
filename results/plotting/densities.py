import numpy as np
from matplotlib import pyplot as plt
from matplotlib.cm import ScalarMappable
from matplotlib.colors import Normalize
import json

import utils

# numnodes = 10
# numdensity = 10
# para_start = 0
# para_end = 10

numnodes = 20
numdensity = 10
para_start = 10
para_end = 20

# getting the correct spacing is really f**ked up; playing with figsize helps


def density():
    main()
    # appendix()


def main():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.67))
    nrows = 11
    nusedrows = 10
    gs = fig.add_gridspec(nrows, 2)
    acs = []
    for i in range(2):
        acs.append(fig.add_subplot(gs[:nusedrows, i]))
    cac = fig.add_subplot(gs[nusedrows:, :])
    map = [0, 3]
    gs.update(wspace=0.1, hspace=0.3)

    cmap = get_cmap()

    draw_images(acs, map, cmap)

    for i in range(2):
        acs[i].grid(False)
        acs[i].set_xlabel("correction density")
    acs[0].set_ylabel("edge density")
    acs[1].set_yticklabels([])
    acs[0].set_title("time cost for time optimal (trivial)")
    acs[1].set_title("space cost for space optimal (approx)")

    draw_colorbar(fig, cac, cmap)

    plt.subplots_adjust(top=0.98, bottom=0.10, left=0.06, right=0.97)
    plt.savefig(f"output/density_main-{numnodes}.pdf")


def appendix():
    fig = plt.figure(figsize=utils.set_size(height_in_width=1.04))
    rowsfactor = 10
    nrows = 2 * rowsfactor + 2
    nusedrows = nrows - 1
    gs = fig.add_gridspec(nrows, 2)
    acs = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        acs.append(fig.add_subplot(gs[i * rowsfactor : rowsfactor * (i + 1), j]))
    cac = fig.add_subplot(gs[nusedrows:, :])
    map = [0, 2, 1, 3]
    gs.update(wspace=0.1, hspace=0.4)

    cmap = plt.get_cmap("viridis").reversed()

    draw_images(acs, map, cmap)

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

    draw_colorbar(fig, cac, cmap)

    plt.subplots_adjust(top=0.96, bottom=0.05, left=0.06, right=0.897)
    plt.savefig(f"output/density_appendix-{numnodes}.pdf")


def get_data(parameter: tuple[float, float]):
    file = (
        f"output/density-numnodes:{numnodes}_numdensities:{numdensity}_"
        f"density:{int(parameter[1])}.json"
    )
    with open(file, "r") as f:
        data = json.load(f)
    return data


def get_cmap():
    return plt.get_cmap("viridis").reversed()


def draw_images(acs, map, cmap):
    len_data = len(map)
    data = [[] for _ in range(len_data)]
    parameters = utils.get_parameters("density")
    for para in parameters[para_start:para_end]:
        dat = get_data(para)["results"]
        for i in range(len_data):
            data[i].append(dat[2 * map[i]])

    for i, dat in enumerate(data):
        acs[i].imshow(dat, origin="lower", cmap=cmap)


def draw_colorbar(fig, cac, cmap):
    fig.colorbar(
        ScalarMappable(norm=Normalize(vmin=0, vmax=1), cmap=cmap),
        cax=cac,
        orientation="horizontal",
    )
    cac.grid(False)
    cac.set_xlabel("cost / num nodes", labelpad=1)
