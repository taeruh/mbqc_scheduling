use std::{
    fs::{self, File},
    ops::Range,
    time::{Duration, Instant},
};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use super::{ConstantDensity, Times};
use crate::{plots::Density, NCPUS, NUM_AVERAGE};

const WALLTIME: u64 = 10; // cf. walltime in scripts/exe_hpc.bash
const TIMEOUT_PER_SINGLE_SHOT_SWEEP: u64 =
    crate::timeout_per_single_shot_sweep(WALLTIME, 30);

const NUM_DENSITIES: usize = 21;
const RANGE: Range<usize> = 1..NUM_DENSITIES + 1;

fn density(multiplier: f64) -> ConstantDensity {
    ConstantDensity::new(multiplier / (NUM_DENSITIES as f64 + 1.))
}

// naive ...
fn timeouts() -> [Duration; NUM_DENSITIES + 1] {
    let mut ret = [Duration::default(); NUM_DENSITIES + 1];
    for ret in ret.iter_mut().skip(1) {
        *ret = Duration::from_nanos(TIMEOUT_PER_SINGLE_SHOT_SWEEP / NUM_DENSITIES as u64);
    }
    ret
}

// gonna sweep over correction density
pub struct Args {
    pub num_nodes: usize,
    pub edge_density_multiplier: f64,
}

pub fn run(args: Args) {
    let full_time = Instant::now();
    tracing_subscriber::fmt::init();

    let size = args.num_nodes;
    let edge_density_multiplier = args.edge_density_multiplier;
    let edge_density = density(edge_density_multiplier);

    let output_file = format!(
        "output/density-numnodes:{}_numdensities:{}_density:{}.json",
        size, NUM_DENSITIES, edge_density_multiplier
    );

    // cf. node plot
    let seed = Pcg64::from_entropy().gen();

    let mut rng = Pcg64::seed_from_u64(seed);
    let mut results: [Vec<f64>; 8] = Default::default();
    let mut time_results = Vec::with_capacity(NUM_DENSITIES);

    let timeouts = timeouts();

    #[cfg(debug_assertions)]
    println!(
        "set:\t\t{:?}\ncalculated:\t{:?}\ntimeouts: {timeouts:?}",
        Duration::from_nanos(TIMEOUT_PER_SINGLE_SHOT_SWEEP),
        timeouts.iter().sum::<Duration>()
    );

    for correction_density_multiplier in RANGE {
        let correction_density = density(correction_density_multiplier as f64);
        println!("{:?}", correction_density);
        let timeout = timeouts[correction_density_multiplier];
        let total_time = Instant::now();
        let (result, times) = super::do_it(
            size,
            edge_density,
            correction_density,
            timeout,
            &mut rng,
            false,
        );
        println!(
            "density={:<3}: total time: {:?}; per shot: {:?} from {:?}",
            correction_density.get(size),
            total_time.elapsed(),
            times.space_optimal_approximated,
            timeout
        );
        for (i, result) in result.iter().enumerate() {
            results[2 * i].push(result.0);
            results[2 * i + 1].push(result.1);
        }
        time_results.push(times);
    }

    let output = Output {
        size,
        num_average: NUM_AVERAGE,
        walltime: WALLTIME,
        ncpus: NCPUS,
        edge_density_multiplier,
        num_density: NUM_DENSITIES,
        seed,
        results,
        time_results,
    };

    fs::create_dir_all("output").unwrap();
    serde_json::to_writer(File::create(output_file).unwrap(), &output).unwrap();

    println!("total time: {:?}", full_time.elapsed());
}

#[derive(Serialize)]
struct Output {
    size: usize,
    num_average: usize,
    walltime: u64,
    ncpus: u16,
    edge_density_multiplier: f64,
    num_density: usize,
    seed: u64,
    results: [Vec<f64>; 8],
    time_results: Vec<Times>,
}
