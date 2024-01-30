#!/usr/bin/bash

jobs=$(seq 1 9)

readarray -t parameters < parameters.dat

mkdir -p output

for p in "${parameters[@]}"; do
  echo $p
  ./target/release/results $p
done
