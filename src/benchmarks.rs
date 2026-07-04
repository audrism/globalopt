pub fn furasn(x: &[f64]) -> f64 {
    let n = x.len();
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = x.iter().map(|&xi| xi * xi - (18.0 * xi).cos()).sum();
    sum * (2.0 / n as f64)
}
