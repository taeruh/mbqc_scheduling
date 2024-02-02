use std::{
    fs::{self, File},
    ops::Range,
    time::{Duration, Instant},
};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use super::ConstantDensity;
use crate::{
    plots::Density, NCPUS, NUM_AVERAGE, TIMEOUT_PER_SINGLE_SHOT_SWEEP, WALLTIME,
};

// depending on the walltime we timeout; do a test run for the first view sizes and ensure
// that there's enough time such that the first few sizes definitely find at least one
// path (run with cargo run --release --no-default-features to see whether timeouts
// occur); important: I'm not sure why, but on our cluster each size may take up to 2.5ms
// longer

const NUM_DENSITIES: usize = 10;
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

    let output_file =
        format!("output/density-size:{size}_density:{edge_density_multiplier}.json",);

    // cf node plot
    let seed = Pcg64::from_entropy().gen();

    let mut rng = Pcg64::seed_from_u64(seed);
    let mut results: [Vec<f64>; 8] = Default::default();

    let timeouts = timeouts();

    #[cfg(debug_assertions)]
    println!(
        "set:\t\t{:?}\ncalculated:\t{:?}\ntimeouts: {timeouts:?}",
        Duration::from_nanos(TIMEOUT_PER_SINGLE_SHOT_SWEEP),
        timeouts.iter().sum::<Duration>()
    );

    for correction_density_multiplier in RANGE {
        let correction_density = density(correction_density_multiplier as f64);
        let timeout = timeouts[size];
        let total_time = Instant::now();
        let (result, averaged_time) =
            super::do_it(size, edge_density, correction_density, timeout, &mut rng);
        println!(
            "density={:<3}: total time: {:?}; per shot: {:?} from {:?}",
            correction_density.get(size),
            total_time.elapsed(),
            averaged_time,
            timeout
        );
        for (i, result) in result.iter().enumerate() {
            results[2 * i].push(result.0);
            results[2 * i + 1].push(result.1);
        }
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
}
