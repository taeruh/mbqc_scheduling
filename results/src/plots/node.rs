use std::{
    fs::{self, File},
    ops::Range,
    time::{Duration, Instant},
};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    plots::{DensityEnum, Times},
    NCPUS, NUM_AVERAGE,
};

// depending on the walltime we timeout; do a test run for the first view sizes and ensure
// that there's enough time such that the first few sizes definitely find at least one
// path (run with cargo run --release --no-default-features to see whether timeouts
// occur); important: I'm not sure why, but on our cluster each size may take up to 5ms
// longer

const WALLTIME: u64 = 192; // cf. walltime in scripts/exe_hpc.bash
// 2h buffer (better to be safe)
const TIMEOUT_PER_SINGLE_SHOT_SWEEP: u64 =
    crate::timeout_per_single_shot_sweep(WALLTIME, 120);

const MAX_SIZE: usize = 45;
const MAX_EXACT_SIZE: usize = 20;

// account for additional exact search; rough (pessimistic; better be safe than sorry)
// guess here; depends on densities; for p_e = 0.5, p_c = 0.5, both reziprocal_square_root
const TIMEOUT_PER_SINGLE_SHOT_EXACT_SWEEP: u64 = 5_000_000_000;

const RANGE: Range<usize> = 2..MAX_SIZE + 1;
const RANGE_LEN: usize = MAX_SIZE - 1;
const REAL_TIMEOUT_PER_SINGLE_SHOT_SWEEP: u64 =
    TIMEOUT_PER_SINGLE_SHOT_SWEEP - TIMEOUT_PER_SINGLE_SHOT_EXACT_SWEEP;

// increase time quadratically (because that's how the everything else scales, more or
// less) with size:
// sum_2^{n} a * x^2 = TIMEOUT_PER_SINGLE_SHOT_SWEEP
// <=> a = TIMEOUT_PER_SINGLE_SHOT_SWEEP / (1/6 n(n+1)(2n+1) - 1)
fn timeouts() -> [Duration; MAX_SIZE + 1] {
    let mut ret = [Duration::default(); MAX_SIZE + 1];
    let a = REAL_TIMEOUT_PER_SINGLE_SHOT_SWEEP as f64
        / (1. / 6. * (MAX_SIZE * (MAX_SIZE + 1) * (2 * MAX_SIZE + 1)) as f64 - 1.);
    for size in RANGE {
        ret[size] = Duration::from_nanos(
            // additional buffer time for starting the job
            ((a * (size as f64).powi(2)).round() as u64).saturating_sub(5_000_000),
        )
    }
    ret
}

// gonna sweep over num_nodes
pub struct Args {
    pub edge_density: f64,
    pub edge_density_type: String,
    pub correction_density: f64,
    pub correction_density_type: String,
}

pub fn run(args: Args) {
    let full_time = Instant::now();
    tracing_subscriber::fmt::init();

    let edge_density = DensityEnum::new(&args.edge_density_type, args.edge_density);
    let correction_density =
        DensityEnum::new(&args.correction_density_type, args.correction_density);

    let output_file = format!(
        "output/node-{}_{}.json",
        serde_json::to_string(&edge_density).unwrap(),
        serde_json::to_string(&correction_density).unwrap()
    )
    .replace(['{', '"', '}'], "");

    // if the number of threads passed to `run` below is 1, then one could replace this
    // seed with the seed in the output_file to reproduce the result, however, the final
    // results are not produced singly threaded, so this is pointless (cf. doc of the run
    // function; not completely pointless, but there will be deviations)
    let seed = Pcg64::from_entropy().gen();

    let mut rng = Pcg64::seed_from_u64(seed);
    let mut results: [Vec<f64>; 16] = Default::default();
    let mut time_results: Vec<Times> = Vec::with_capacity(RANGE_LEN);

    let timeouts = timeouts();

    #[cfg(debug_assertions)]
    println!(
        "set:\t\t{:?}\ncalculated:\t{:?}\ntimeouts: {timeouts:?}",
        Duration::from_nanos(REAL_TIMEOUT_PER_SINGLE_SHOT_SWEEP),
        timeouts.iter().sum::<Duration>()
    );

    for size in RANGE {
        let timeout = timeouts[size];
        let total_time = Instant::now();
        let (result, times) = super::do_it(
            size,
            edge_density,
            correction_density,
            timeout,
            &mut rng,
            size <= MAX_EXACT_SIZE,
        );
        println!(
            "size={size:<3}: total time: {:?}; per shot: {:?} from {:?};; full time: \
             {:?}",
            total_time.elapsed(),
            times.space_optimal_approximated,
            timeout,
            times.full
        );
        // results.push((size, result));
        for (i, result) in result.iter().enumerate() {
            results[2 * i].push(result.0);
            results[2 * i + 1].push(result.1);
        }
        time_results.push(times);
    }

    let output = Output {
        max_size: MAX_SIZE,
        num_average: NUM_AVERAGE,
        walltime: WALLTIME,
        ncpus: NCPUS,
        edge_density,
        correction_density,
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
    max_size: usize,
    num_average: usize,
    walltime: u64,
    ncpus: u16,
    edge_density: DensityEnum,
    correction_density: DensityEnum,
    seed: u64,
    results: [Vec<f64>; 16],
    time_results: Vec<Times>,
}
