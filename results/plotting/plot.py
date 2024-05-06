#!/usr/bin/env python

import utils

# output = "output"
output = "data"


def main():
    utils.paper_setup()
    node()
    density()
    runtime()


def density():
    import densities

    densities.density()


def node():
    import nodes

    nodes.node()


def runtime():
    import runtimes

    runtimes.runtime()


if __name__ == "__main__":
    main()
