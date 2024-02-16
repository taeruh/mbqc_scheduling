from matplotlib import pyplot as plt
from matplotlib.cm import ScalarMappable
from matplotlib.colors import Normalize
import math
import utils
import densities, nodes


def runtime():
    appendix()


def appendix():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.50))
    ncols_factor = 8
    ncols = 2 * ncols_factor + 3
    nusedcols = ncols - 1
    gs = fig.add_gridspec(1, ncols)
    acs = []
    for i in range(2):
        acs.append(
            fig.add_subplot(
                gs[0, i * ncols_factor + i * 2 : (i + 1) * ncols_factor + i * 2]
            )
        )
    cac = fig.add_subplot(gs[0, nusedcols:])
    gs.update(wspace=0.3)

    data = get_data()

    labels = ["space optimal", "full"]

    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [colors[3], colors[2]]
    linestyles = ["dotted", "solid"]

    node_data = data["node"]

    for i, dat in enumerate([node_data["space_optimal"], node_data["full"]]):
        acs[0].plot(
            range(2, len(dat) + 2),
            dat,
            label=labels[i],
            color=colors[i],
            linestyle=linestyles[i],
        )
    acs[0].set_yscale("log")
    acs[0].set_ylabel("runtime [nanoseconds]")
    acs[0].set_xlabel("num nodes")

    density_data = data["density"]

    cmap = densities.get_cmap()

    acs[1].imshow(density_data["space_optimal"], origin="lower", cmap=cmap)
    acs[1].set_xlabel("correction density", labelpad=17)
    acs[1].set_ylabel("edge density", labelpad=22)
    acs[1].set_title("normalized runtime (log; approx)", pad=6)
    acs[1].set_xticks([])
    acs[1].set_yticks([])
    densities.xticks(acs[1], -0.6)
    densities.yticks(acs[1], -0.1)

    utils.subplotlabel(acs[0], "a", -0.10, 1.05)
    utils.subplotlabel(acs[1], "b", -0.10, 1.05)

    draw_colorbar(fig, cac, cmap)

    handles, labels = acs[0].get_legend_handles_labels()
    acs[0].legend(handles, labels, loc="upper left", labelspacing=0.25)

    plt.subplots_adjust(top=0.93, bottom=0.11, left=0.07, right=0.95)
    plt.savefig(f"output/runtime_appendix.pdf")


def get_data():
    node_parameters = utils.get_parameters("node")
    runtime = nodes.get_data(node_parameters[0])["time_results"]

    space_optimal = []
    full = []
    keys = ["space_optimal_approximated", "full"]

    for time in runtime:
        for dat, key in zip([space_optimal, full], keys):
            t = time.get(key)
            if t is None:
                continue
            dat.append(t["secs"] * 1000000000 + t["nanos"])

    node_data = {
        "space_optimal": space_optimal,
        "full": full,
    }

    density_parameters = utils.get_parameters("density")[:10]

    # density_parameters = [
    #     density_parameters[1],
    #     density_parameters[2],
    #     density_parameters[3],
    #     density_parameters[5],
    #     density_parameters[7],
    #     density_parameters[9],
    # ]

    space_optimal = []
    keys = ["space_optimal_approximated"]
    for para in density_parameters:
        runtime = densities.get_data(para)["time_results"]
        space_optimal_ = []
        for time in runtime[1:]:
            for dat, key in zip([space_optimal_], keys):
                t = time.get(key)
                if t is None:
                    continue
                dat.append(math.log(t["secs"] * 1000000000 + t["nanos"], 2))
                # dat.append(t["secs"] * 1000000000 + t["nanos"])
        space_optimal.append(space_optimal_)

    density_data = {"space_optimal": space_optimal}

    return {"node": node_data, "density": density_data}


def draw_colorbar(fig, cac, cmap):
    fig.colorbar(
        ScalarMappable(norm=Normalize(vmin=0, vmax=1), cmap=cmap),
        cax=cac,
        orientation="vertical",
    )
    cac.grid(False)
