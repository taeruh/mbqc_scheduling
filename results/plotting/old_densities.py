from matplotlib import pyplot as plt
import json

import utils


numnodes = 20
numdensity = 20
# numdensity = 10

# getting the correct spacing is really f**ked up; playing with figsize helps

xlabel = r"correction density $p_c$"
ylabel = r"edge density $p_e$"


def density():
    main()
    # appendix()


def main():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.67))
    nrows = 11
    nusedrows = nrows - 1
    gs = fig.add_gridspec(nrows, 2)
    acs = []
    for i in range(2):
        acs.append(fig.add_subplot(gs[:nusedrows, i]))
    cac = fig.add_subplot(gs[nusedrows:, :])
    map = [0, 3]
    gs.update(wspace=0.1, hspace=0.3)

    cmap = get_cmap()

    im = draw_images(acs, map, cmap)

    for i in range(2):
        acs[i].grid(False)
        acs[i].set_xticks([])
        acs[i].set_yticks([])
        acs[i].set_xlabel(xlabel, labelpad=16)
        xticks(acs[i], -0.06)

    acs[0].set_ylabel(ylabel, labelpad=20)
    yticks(acs[0], -0.08)
    acs[0].set_title(r"time cost $\mathrm{tc}(S_{tt})$")
    acs[1].set_title(r"space cost $\mathrm{sc}(S_{sa})$")

    utils.subplotlabel(acs[0], "a", -0.06, 1.06)
    utils.subplotlabel(acs[1], "b", -0.06, 1.06)

    fig.colorbar(
        im,
        cax=cac,
        orientation="horizontal",
    )
    cac.grid(False)
    cac.set_xlabel(r"time cost \hspace{1em}|\hspace{1em} space cost", labelpad=1)

    plt.subplots_adjust(top=0.98, bottom=0.10, left=0.06, right=0.97)
    plt.savefig(f"output/density_main-{numnodes}.pdf")


def appendix():
    # fig = plt.figure(figsize=utils.set_size(height_in_width=1.04))
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.67))
    # rowsfactor = 10
    # nrows = 2 * rowsfactor + 2
    # nusedrows = nrows - 1
    nrows = 11
    nusedrows = nrows - 1
    gs = fig.add_gridspec(nrows, 2)
    acs = []
    # for i, j in [(i, j) for i in range(2) for j in range(2)]:
    # acs.append(fig.add_subplot(gs[i * rowsfactor : rowsfactor * (i + 1), j]))
    for i in range(2):
        acs.append(fig.add_subplot(gs[:nusedrows, i]))
    cac = fig.add_subplot(gs[nusedrows:, :])
    # map = [0, 2, 1, 3]
    map = [1, 2]
    # gs.update(wspace=0.1, hspace=0.4)
    gs.update(wspace=0.1, hspace=0.3)

    cmap = plt.get_cmap("viridis").reversed()

    im = draw_images(acs, map, cmap)

    # for i in range(4):
    for i in range(2):
        acs[i].grid(False)
        acs[i].set_xticks([])
        acs[i].set_yticks([])
        # if i > 1:
        acs[i].set_xlabel(xlabel, labelpad=16)
        xticks(acs[i], -0.06)
        if i % 2 == 0:
            acs[i].set_ylabel(ylabel, labelpad=20)
            yticks(acs[i], -0.08)
        # if i == 0:
        #     acs[i].set_title("time optimal")
        # if i == 1:
        #     acs[i].set_title("space optimal (approx)")
        #     acs[i].text(1.05, 0.5, "time cost", transform=acs[i].transAxes, rotation=45)
        # if i == 3:
        #     acs[i].text(
        #         1.05, 0.5, "space cost", transform=acs[i].transAxes, rotation=45
        #     )
    acs[0].set_title(r"space cost $\mathrm{sc}(S_{tt})$")
    acs[1].set_title(r"time cost $\mathrm{tc}(S_{sa})$")

    utils.subplotlabel(acs[0], "a", -0.06, 1.06)
    utils.subplotlabel(acs[1], "b", -0.06, 1.06)

    fig.colorbar(
        im,
        cax=cac,
        orientation="horizontal",
    )
    cac.grid(False)
    cac.set_xlabel(r"space cost \hspace{1em}|\hspace{1em} time cost", labelpad=1)

    # plt.subplots_adjust(top=0.96, bottom=0.05, left=0.06, right=0.897)
    plt.subplots_adjust(top=0.98, bottom=0.10, left=0.06, right=0.97)
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
