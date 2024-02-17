from matplotlib import pyplot as plt
from matplotlib.cm import ScalarMappable
from matplotlib.colors import Normalize, LogNorm
import math
import utils
import densities, nodes


def runtime():
    appendix()


xllabel = r"number of nodes $\abs{G}$"
xrlabel = r"correction density $p_c$"
yrlabel = r"edge density $p_e$"

def appendix():
    fig = plt.figure(figsize=utils.set_size(height_in_width=0.5))
    ncols_factor = 8
    ncols = 2 * ncols_factor + 5
    nusedcols = ncols - 1
    gs = fig.add_gridspec(1, ncols)
    acs = []
    for i in range(2):
        acs.append(
            fig.add_subplot(
                gs[0, i * ncols_factor + i * 2 : (i + 1) * ncols_factor + i * 4]
            )
        )
    cac = fig.add_subplot(gs[0, nusedcols:])
    gs.update(wspace=0.7)

    data = get_data()

    labels = ["approx. search", "exact optimization"]

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
    acs[0].set_ylabel("run-time [nanoseconds]")
    acs[0].set_xlabel(xllabel)

    density_data = data["density"]

    cmap = densities.get_cmap()

    print(density_data["min"], density_data["max"])
    im = acs[1].imshow(
        density_data["space_optimal"],
        origin="lower",
        cmap=cmap,
        norm=LogNorm(vmin=density_data["min"], vmax=density_data["max"]),
    )
    acs[1].set_xlabel(xrlabel, labelpad=17)
    acs[1].set_ylabel(yrlabel, labelpad=22)
    acs[1].set_title("run-time [nanoseconds] (approx. search)", pad=6)
    acs[1].set_xticks([])
    acs[1].set_yticks([])
    densities.xticks(acs[1], -0.06)
    densities.yticks(acs[1], -0.1)

    utils.subplotlabel(acs[0], "a", -0.10, 1.05)
    utils.subplotlabel(acs[1], "b", -0.10, 1.05)

    fig.colorbar(
        im,
        cax=cac,
        orientation="vertical",
    )
    cac.grid(False)

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

    density_parameters = utils.get_parameters("density")

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
    min = 999 * 60 * 1000000000  # 999 hours
    max = 0
    for para in density_parameters:
        runtime = densities.get_data(para)["time_results"]
        space_optimal_ = []
        for time in runtime[0:]:
            for dat, key in zip([space_optimal_], keys):
                t = time.get(key)
                if t is None:
                    continue
                nanos = t["secs"] * 1000000000 + t["nanos"]
                if nanos < min:
                    min = nanos
                if nanos > max:
                    max = nanos
                # dat.append(math.log(nanos, 2))
                dat.append(nanos)
        space_optimal.append(space_optimal_)

    density_data = {"space_optimal": space_optimal, "min": min, "max": max}

    return {"node": node_data, "density": density_data}
