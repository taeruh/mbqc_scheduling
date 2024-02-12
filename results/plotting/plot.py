#!/usr/bin/env python

import utils


def main():
    utils.paper_setup()
    node()
    # density()


def density():
    import densities

    densities.density()


def node():
    import nodes

    nodes.node()


if __name__ == "__main__":
    main()
