use globalopt::{bayes1, furasn, Bayes1Config};

fn main() {
    let a = [-0.25, -0.125];
    let b = [0.5, 0.625];

    let result = bayes1(
        &a,
        &b,
        Bayes1Config {
            evaluations: 200,
            initial_points: 20,
        },
        furasn,
    )
    .expect("BAYES1 optimization should succeed");

    println!("BAYES1 best f: {:.8}", result.best_f);
    println!("BAYES1 best x: {:?}", result.best_x);
    println!("BAYES1 best iteration: {}", result.best_iter);
}
