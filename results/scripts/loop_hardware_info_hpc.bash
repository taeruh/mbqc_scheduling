#!/usr/bin/bash

# i3node01 and c3node03 are privat

nodes="02 03 04 05 06 07 08 09 10 11 12"

for node in $nodes; do
  qsub -l nodes=hpcnode$node -v node="$node" scripts/hardware_info_hpc.bash
done
