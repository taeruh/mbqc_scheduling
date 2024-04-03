use plots::Args;

const NCPUS: u16 = 9;
// const NCPUS: u16 = 1;
// const NCPUS: u16 = 30; // cf. ncpus in scripts/exe_hpc.bash

// const NUM_AVERAGE: usize = 3000;
const NUM_AVERAGE: usize = 1000;
// const NUM_AVERAGE: usize = 50;

/// timeout per single shot sweep in nano seconds; walltime is in hours, buffer in minutes
// 5min buffer for timeouts (better to be safe)
const fn timeout_per_single_shot_sweep(walltime: u64, buffer: u64) -> u64 {
    (walltime * 3600 - buffer * 60) * 1_000_000_000 / NUM_AVERAGE as u64
}

pub fn run(args: Args) {
    match args {
        Args::Node(args) => plots::node::run(args),
        Args::Density(args) => plots::density::run(args),
    }
}

pub mod cli;
pub mod plots;
