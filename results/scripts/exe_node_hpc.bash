#!/usr/bin/bash

# don't define anything before the PBS options

#PBS -m eba
#PBS -M jannis.ruh@student.uts.edu.au
#PBS -N mbqc_scheduling_nodes

#PBS -l ncpus=30
#PBS -l mem=5GB
#PBS -l walltime=192:00:00

# this is relative to the final workdir which is ./=${PBS_O_WORKDIR}, so we don't have
# to move it from the scratch
#PBS -e ./log/
#PBS -o ./log/

cd ${PBS_O_WORKDIR}
mkdir -p log
mkdir -p output
readarray -t parameters < "parameters/node.dat"

scratch="/scratch/${USER}_${PBS_JOBID%.*}"
mkdir -p ${scratch}/output
cp target/release/results ${scratch}

cd ${scratch}

./results node ${parameters[0]}
# NOTE: `cd ${PBS_O_WORKDIR}; mv ${scratch}/output/* output` doesn't work; it's the wild
# card that makes problems in this case, but I don't know why (maybe the ${scratch} name
# is too weird)?
mv output/* ${PBS_O_WORKDIR}/output/

cd ${PBS_O_WORKDIR}
rm -rf ${scratch}
