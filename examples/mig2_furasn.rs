use globalopt::{furasn, mig2, Mig2Config};

fn main() {
    let a = [-0.25, -0.125];
    let b = [0.5, 0.625];

    let result = mig2(&a, &b, Mig2Config { evaluations: 200 }, furasn)
        .expect("MIG2 optimization should succeed");

    println!("MIG2 best f: {:.8}", result.best_f);
    println!("MIG2 best x: {:?}", result.best_x);
    println!("MIG2 best iteration: {}", result.best_iter);
}
