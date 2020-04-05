use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::merge_sort_adaptive;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

const PROBLEM_SIZE: u32 = 100_000_000;
fn main() {
    let mut input = (0..PROBLEM_SIZE).rev().collect::<Vec<u32>>();
    let solution = (0..PROBLEM_SIZE).collect::<Vec<u32>>();
    input.shuffle(&mut thread_rng());
    #[cfg(feature = "logs")]
    {
        let thresholds: Vec<usize> = vec![
            100_000_000,
            50_000_000,
            25_000_000,
            12_500_000,
            6_250_000,
            3_125_000,
            1_562_500,
            781_250,
            390_625,
            195_313,
            97_657,
            48_828,
        ];
        thresholds.into_iter().for_each(|threshold| {
            let p = ThreadPoolBuilder::new()
                .num_threads(16)
                .build()
                .expect("builder failed");
            let log = p
                .logging_install(|| merge_sort_adaptive(&mut input, threshold))
                .1;
            log.save_svg(format!("join_policy_sort_log_{}.svg", threshold))
                .expect("saving svg file failed");
        });
    }
    #[cfg(not(feature = "logs"))]
    {
        let thresholds: Vec<usize> = vec![
            100_000_000,
            50_000_000,
            25_000_000,
            12_500_000,
            6_250_000,
            3_125_000,
            1_562_500,
            781_250,
            390_625,
            195_313,
            97_657,
            48_828,
        ];
        let tp = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("cant build pool");
        tp.install(|| {
            thresholds.into_iter().for_each(|threshold| {
                merge_sort_adaptive(&mut input, threshold);
            });
        });
        assert_eq!(input, solution);
    }
}
