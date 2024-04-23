from matplotlib import pyplot as plt

# from matplotlib.cm import ScalarMappable
# from matplotlib.colors import Normalize
import json

import utils


numnodes = 20
numdensity = 20
# numnodes = 13
# numdensity = 13

# getting the correct spacing is really f**ked up; playing with figsize helps

xlabel = r"correction density $p_c$"
ylabel = r"edge density $p_e$"


def density():
    fig = plt.figure(figsize=utils.set_size(height_in_width=1.04))
    # fig = plt.figure(figsize=utils.set_size(height_in_width=0.67))
    rowsfactor = 10
    nrows = 2 * rowsfactor + 2
    nusedrows = nrows - 1
    # nrows = 11
    # nusedrows = nrows - 1
    gs = fig.add_gridspec(nrows, 2)
    acs = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        acs.append(fig.add_subplot(gs[i * rowsfactor : rowsfactor * (i + 1), j]))
    cac = fig.add_subplot(gs[nusedrows:, :])
    map = [0, 1, 2, 3]
    gs.update(wspace=0.1, hspace=0.4)
    gs.update(wspace=0.1, hspace=0.3)

    cmap = get_cmap()

    im = draw_images(acs, map, cmap)

    figlabels = ["a", "b", "c", "d"]

    def rowlabel(i, a, b, c):
        acs[i].text(1.02, 0.54, a, transform=acs[i].transAxes, rotation=45)
        acs[i].text(1.02, 0.46, b, transform=acs[i].transAxes, rotation=45)
        acs[i].text(1.06, 0.42, c, transform=acs[i].transAxes, rotation=45)

    for i in range(4):
        acs[i].grid(False)
        acs[i].set_xticks([])
        acs[i].set_yticks([])
        utils.subplotlabel(acs[i], figlabels[i], -0.045, 1.035)
        if i > 1:
            acs[i].set_xlabel(xlabel, labelpad=16)
            xticks(acs[i], -0.06)
        if i % 2 == 0:
            acs[i].set_ylabel(ylabel, labelpad=20)
            yticks(acs[i], -0.08)
        if i == 1:
            acs[i].set_title("space optimal (approx)")
            rowlabel(
                i, "time optimal,", "trivial schedule", r"$S_{\text{trivial,time}}$"
            )
        if i == 3:
            rowlabel(
                i, "space optimal,", "appr. schedule", r"$S_{\text{approx,space}}$"
            )

    acs[0].set_title(r"time cost $\mathrm{tc}(S)$")
    acs[1].set_title(r"space cost $\mathrm{sc}(S)$")

    fig.colorbar(
        im,
        cax=cac,
        orientation="horizontal",
    )
    cac.grid(False)
    # cac.set_xlabel(r"time cost \hspace{2.5em}|\hspace{2.5em} space cost", labelpad=1)
    cac.set_xlabel(r"time cost \& space cost (cf. Def. 7)", labelpad=1)

    plt.subplots_adjust(top=0.97, bottom=0.05, left=0.07, right=0.885)
    plt.savefig(f"output/density-{numnodes}.pdf")


def get_data(parameter: tuple[float, float]):
    file = (
        f"output/density-numnodes:{numnodes}_numdensities:{numdensity}_"
        f"density:{int(parameter[1])}.json"
    )
    with open(file, "r") as f:
        data = json.load(f)
    return data


def get_cmap():
    return plt.get_cmap("turbo").reversed()  # "jet"


def draw_images(acs, map, cmap):
    len_data = len(map)
    data = [[] for _ in range(len_data)]
    parameters = utils.get_parameters("density")

    # get global min and max, to set the same color scale for all images
    min = numnodes
    max = 0

    for para in parameters:
        dat = get_data(para)["results"]
        for i in range(len_data):
            data[i].append(dat[2 * map[i]])
            for d in dat[2 * map[i]]:
                if d < min:
                    min = d
                if d > max:
                    max = d

    for i, dat in enumerate(data):
        if i == 0:
            im = acs[i].imshow(dat, origin="lower", cmap=cmap, vmin=min, vmax=max)
        else:
            acs[i].imshow(dat, origin="lower", cmap=cmap, vmin=min, vmax=max)

    return im  # pyright: ignore


def xticks(ac, shift):
    ac.text(
        0.0,
        shift,
        "0.0",
        transform=ac.transAxes,
        horizontalalignment="left",
    )
    ac.text(
        0.5,
        shift,
        "0.5",
        transform=ac.transAxes,
        horizontalalignment="center",
    )
    ac.text(
        1.0,
        shift,
        "1.0",
        transform=ac.transAxes,
        horizontalalignment="right",
    )


def yticks(ac, shift):
    ac.text(
        shift,
        0.0,
        "0.0",
        transform=ac.transAxes,
        verticalalignment="bottom",
    )
    ac.text(
        shift,
        0.5,
        "0.5",
        transform=ac.transAxes,
        verticalalignment="center",
    )
    ac.text(
        shift,
        1.0,
        "1.0",
        transform=ac.transAxes,
        verticalalignment="top",
    )
