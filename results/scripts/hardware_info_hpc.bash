#!/usr/bin/bash

#PBS -m eba
#PBS -M jannis.ruh@student.uts.edu.au
#PBS -N get_hardware_info

#PBS -l ncpus=1
#PBS -l mem=1GB
#PBS -l walltime=1:00

#PBS -e ./log/
#PBS -o ./log/

cd ${PBS_O_WORKDIR}
mkdir -p log
mkdir -p output

scratch="/scratch/${USER}_${PBS_JOBID%.*}"
mkdir -p ${scratch}/output

cd ${scratch}

lscpu > "output/hardware_info_hpcnode_${node}.txt"

mv output/* ${PBS_O_WORKDIR}/output/

cd ${PBS_O_WORKDIR}
rm -rf ${scratch}
