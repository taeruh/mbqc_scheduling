#!/usr/bin/env python

from matplotlib import pyplot as plt
import json


def main():
    parameters = get_parameters()

    para = parameters[0]
    data = get_data(para)

    y = []
    x = []
    for dat in data["results"]:
        x.append(dat[0])
        y.append(dat[1][0][0])

    plt.plot(x, y)
    plt.show()


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
