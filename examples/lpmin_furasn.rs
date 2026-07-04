use globalopt::{furasn, lpmin, LpminConfig};

fn main() {
    let a = [-0.25, -0.125];
    let b = [0.5, 0.625];

    let result = lpmin(
        &a,
        &b,
        LpminConfig {
            analysis_evals: 50,
            search_evals: 200,
            variable_order: None,
        },
        furasn,
    )
    .expect("LPMIN optimization should succeed");

    println!("LPMIN best f: {:.8}", result.best_f);
    println!("LPMIN best x: {:?}", result.best_x);
    println!("LPMIN best iteration: {}", result.best_iter);
}
