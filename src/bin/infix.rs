use rayon_adaptive::*;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::ThreadPoolBuilder;
const NUM_THREADS: usize = 2;
const SIZE: u64 = 10_000_000;
fn main() {
    #[cfg(feature = "logs")]
    {
        let pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("Pool creation failed");

        pool.compare()
            .runs_number(100)
            .attach_algorithm_nodisplay_with_setup(
                "sequential",
                || vec_gen(SIZE),
                |vec| {
                    solver_seq(&vec);
                    vec
                },
            )
            .attach_algorithm_with_setup(
                "fully adaptive",
                || vec_gen(SIZE),
                |vec| {
                    solver_fully_adaptive(&vec);
                    vec
                },
            )
            .attach_algorithm_with_setup(
                "adaptive",
                || vec_gen(SIZE),
                |vec| {
                    solver_adaptive(&vec, Default::default());
                    vec
                },
            )
            .generate_logs(format!(
                "comparisons_{}mil_{}threads.html",
                SIZE / (1e6 as u64),
                NUM_THREADS
            ))
            .expect("comparison failed");
    }
    #[cfg(not(feature = "logs"))]
    {
        ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build_global()
            .expect("Pool creation failed");

        let random_expression = vec_gen(SIZE);
        let answer = solver_seq(&random_expression);
        let adapt_ans = solver_adaptive(&random_expression, Default::default());
        let parsplit_ans = solver_par_split(&random_expression);
        let parfold_ans = solver_par_fold(&random_expression);
        let adaptfold_ans = solver_fully_adaptive(&random_expression);
        assert_eq!(answer, adapt_ans);
        assert_eq!(answer, parsplit_ans);
        assert_eq!(answer, parfold_ans);
        assert_eq!(answer, adaptfold_ans);
    }
}
