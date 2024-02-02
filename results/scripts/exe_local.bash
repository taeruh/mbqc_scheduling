#!/usr/bin/bash

# task="node"
# jobs=$(seq 1 9)

task="density"
jobs=$(seq 1 10)


readarray -t parameters < "parameters/${task}.dat"

mkdir -p output

for p in "${parameters[@]}"; do
  echo $p
  ./target/release/results "$task" $p
done
