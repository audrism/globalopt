pub fn furasn(x: &[f64]) -> f64 {
    let n = x.len();
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = x.iter().map(|&xi| xi * xi - (18.0 * xi).cos()).sum();
    sum * (2.0 / n as f64)
}

pub fn fush5(x: &[f64]) -> f64 {
    shekel_family(x, &A_SHEKEL_5, &C_SHEKEL_5)
}

pub fn fush7(x: &[f64]) -> f64 {
    shekel_family(x, &A_SHEKEL_7, &C_SHEKEL_7)
}

pub fn fush10(x: &[f64]) -> f64 {
    shekel_family(x, &A_SHEKEL_10, &C_SHEKEL_10)
}

pub fn fuhar3(x: &[f64]) -> f64 {
    hartmann_family(x, &ALFA_HAR3, &C_HAR, &P_HAR3)
}

pub fn fuhar6(x: &[f64]) -> f64 {
    hartmann_family(x, &ALFA_HAR6, &C_HAR, &P_HAR6)
}

pub fn fubran(x: &[f64]) -> f64 {
    if x.len() != 2 {
        return f64::INFINITY;
    }
    let x1 = x[0];
    let x2 = x[1];
    (x2 - 0.1292 * x1 * x1 + 1.59155 * x1 - 6.0).powi(2) + 9.60211 * x1.cos() + 10.0
}

pub fn fugold(x: &[f64]) -> f64 {
    if x.len() != 2 {
        return f64::INFINITY;
    }
    let x1 = x[0];
    let x2 = x[1];
    let x3 = x1 * x1;
    let x4 = x2 * x2;
    let x5 = x1 * x2;
    (1.0 + (x1 + x2 + 1.0).powi(2) * (19.0 - 14.0 * x1 + 3.0 * x3 - 14.0 * x2 + 6.0 * x5 + 3.0 * x4))
        * (30.0 + (2.0 * x1 - 3.0 * x2).powi(2) * (18.0 - 32.0 * x1 + 12.0 * x3 + 48.0 * x2 - 36.0 * x5 + 27.0 * x4))
}

fn shekel_family(x: &[f64], a: &[[f64; 4]], c: &[f64]) -> f64 {
    if x.len() != 4 {
        return f64::INFINITY;
    }
    let mut f = 0.0;
    for (row, &ci) in a.iter().zip(c.iter()) {
        let mut f1 = ci;
        for j in 0..4 {
            let d = x[j] - row[j];
            f1 += d * d;
        }
        f -= 1.0 / f1;
    }
    f
}

fn hartmann_family(x: &[f64], alfa: &[[f64; 4]], c: &[f64; 4], p: &[[f64; 4]]) -> f64 {
    if x.len() != alfa.len() {
        return f64::INFINITY;
    }
    let n = x.len();
    let mut f = 0.0;
    for i in 0..4 {
        let mut f1 = 0.0;
        for (j, xj) in x.iter().enumerate().take(n) {
            let d = *xj - p[j][i];
            f1 -= alfa[j][i] * d * d;
        }
        f -= c[i] * f1.exp();
    }
    f
}

const A_SHEKEL_5: [[f64; 4]; 5] = [
    [4.0, 4.0, 4.0, 4.0],
    [1.0, 1.0, 1.0, 1.0],
    [8.0, 8.0, 8.0, 8.0],
    [6.0, 6.0, 6.0, 6.0],
    [3.0, 7.0, 3.0, 7.0],
];

const C_SHEKEL_5: [f64; 5] = [0.1, 0.2, 0.2, 0.4, 0.4];

const A_SHEKEL_7: [[f64; 4]; 7] = [
    [4.0, 4.0, 4.0, 4.0],
    [1.0, 2.0, 1.0, 2.0],
    [8.0, 5.0, 8.0, 5.0],
    [6.0, 4.0, 6.0, 4.0],
    [3.0, 1.0, 3.0, 1.0],
    [2.0, 8.0, 2.0, 8.0],
    [5.0, 7.0, 5.0, 7.0],
];

const C_SHEKEL_7: [f64; 7] = [0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3];

const A_SHEKEL_10: [[f64; 4]; 10] = [
    [4.0, 4.0, 4.0, 4.0],
    [1.0, 2.0, 1.0, 2.0],
    [8.0, 5.0, 8.0, 5.0],
    [6.0, 8.0, 6.0, 8.0],
    [3.0, 7.0, 3.0, 7.0],
    [2.0, 9.0, 2.0, 9.0],
    [5.0, 4.0, 5.0, 4.0],
    [8.0, 1.0, 8.0, 1.0],
    [6.0, 2.0, 6.0, 2.0],
    [7.0, 3.6, 7.0, 3.6],
];

const C_SHEKEL_10: [f64; 10] = [0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3, 0.7, 0.5, 0.5];

const ALFA_HAR3: [[f64; 4]; 3] = [[3.0, 0.1, 3.0, 0.1], [10.0, 10.0, 10.0, 10.0], [30.0, 35.0, 30.0, 35.0]];
const P_HAR3: [[f64; 4]; 3] = [
    [0.3689, 0.4699, 0.1091, 0.03815],
    [0.1170, 0.4387, 0.8732, 0.5743],
    [0.2673, 0.7470, 0.5547, 0.8828],
];

const ALFA_HAR6: [[f64; 4]; 6] = [
    [10.0, 0.05, 3.0, 17.0],
    [3.0, 10.0, 3.5, 8.0],
    [17.0, 17.0, 1.7, 0.05],
    [3.5, 0.1, 10.0, 10.0],
    [1.7, 8.0, 17.0, 0.1],
    [8.0, 14.0, 8.0, 14.0],
];
const P_HAR6: [[f64; 4]; 6] = [
    [0.1312, 0.2329, 0.2348, 0.4047],
    [0.1696, 0.4135, 0.1451, 0.8828],
    [0.5569, 0.8307, 0.3522, 0.8732],
    [0.0124, 0.3736, 0.2883, 0.5743],
    [0.8283, 0.1004, 0.3047, 0.1091],
    [0.5886, 0.9991, 0.6650, 0.0381],
];

const C_HAR: [f64; 4] = [1.0, 1.2, 3.0, 3.2];
