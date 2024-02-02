use plots::Args;

const NCPUS: u16 = 10;
// const NCPUS: u16 = 30; // cf. ncpus in scripts/exe_hpc.bash
const WALLTIME: u64 = 60 * 3600; // cf. walltime in scripts/exe_hpc.bash
const NUM_AVERAGE: usize = 1500;

// 5min buffer for timeouts (better to be safe), and in nano seconds
const TIMEOUT_PER_SINGLE_SHOT_SWEEP: u64 =
    (WALLTIME - 5 * 60) * 1_000_000_000 / NUM_AVERAGE as u64;

pub fn run(args: Args) {
    match args {
        Args::Node(args) => plots::node::run(args),
        Args::Density(args) => plots::density::run(args),
    }
}

pub mod cli;
pub mod plots;
