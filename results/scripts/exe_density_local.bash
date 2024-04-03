#!/usr/bin/bash

mkdir -p output
readarray -t parameters < "parameters/density.dat"

for para in "${parameters[@]}"; do
  ./target/release/results density ${para}
done
