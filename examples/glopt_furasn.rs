use globalopt::{furasn, glopt, GloptConfig};

fn main() {
    let a = [-0.25, -0.125];
    let b = [0.5, 0.625];

    let result = glopt(
        &a,
        &b,
        GloptConfig {
            evaluations: 400,
            initial_points: 80,
            local_trials: 6,
            shrink: 0.92,
        },
        furasn,
    )
    .expect("GLOPT optimization should succeed");

    println!("GLOPT best f: {:.8}", result.best_f);
    println!("GLOPT best x: {:?}", result.best_x);
    println!("GLOPT best iteration: {}", result.best_iter);
}
