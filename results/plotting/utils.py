from matplotlib import pyplot as plt


def get_parameters(task):
    parameters = []
    with open(f"parameters/{task}.dat", "r") as f:
        for pair in f.read().splitlines():
            pair = pair.split(" ")
            parameters.append((float(pair[0]), float(pair[1])))
    return parameters


def paper_setup():
    plt.style.use(
        [
            "./plotting/styles/ownstandard.mplstyle",
            "./plotting/styles/ownlatex.mplstyle",
        ]
    )
    plt.rcParams.update(
        {
            "figure.figsize": [*set_size()],
            "font.size": 10,
            "lines.linewidth": 1.5,
        }
    )


# get default with \the\textwidth
def set_size(width_in_pt=510.0, height_in_width=1.0, scale=1.0):
    width_in_in = width_in_pt * scale / 72.27
    return (width_in_in, width_in_in * height_in_width)


def subplotlabel(ac, label, x=-0.138, y=1.01):
    labels = {
        "a": r"(a)",
        "b": r"(b)",
        "c": r"(c)",
        "d": r"(d)",
    }
    ac.text(x, y, labels[label], ha="left", va="center", transform=ac.transAxes)
