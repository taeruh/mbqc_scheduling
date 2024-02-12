from matplotlib import pyplot as plt
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


def density():
    fig = plt.figure()
    gs = fig.add_gridspec(2, 2)
    acs = []
    for i, j in [(i, j) for i in range(2) for j in range(2)]:
        acs.append(fig.add_subplot(gs[i, j]))
    map = [0, 2, 1, 3]
    gs.update(hspace=0.04, wspace=-0.4)

    data = [[] for _ in range(4)]

    parameters = utils.get_parameters("density")
    for para in parameters[para_start:para_end]:
        dat = get_data(para)["results"]
        for j in range(4):
            data[map[j]].append(dat[2 * j])

    cmap = plt.get_cmap("viridis").reversed()
    images = []
    for i, dat in enumerate(data):
        images.append(acs[i].imshow(dat, origin="lower", cmap=cmap))

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

    plt.savefig(f"output/{density}-{numnodes}.pdf")


def get_data(parameter: tuple[float, float]):
    file = (
        f"output/density-numnodes:{numnodes}_numdensities:{numdensity}_"
        f"density:{int(parameter[1])}.json"
    )
    with open(file, "r") as f:
        data = json.load(f)
    return data
