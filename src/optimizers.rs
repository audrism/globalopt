use thiserror::Error;

use crate::lptau::{lp_tau_point, AtsGenerator};

#[derive(Debug, Error)]
pub enum OptError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct OptResult {
    pub best_x: Vec<f64>,
    pub best_f: f64,
    pub evals: usize,
    pub best_iter: usize,
    pub points: Vec<Vec<f64>>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct AnalResult {
    pub variable_order: Vec<usize>,
    pub influence_scores: Vec<f64>,
    pub samples: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Mig2Config {
    pub evaluations: usize,
}

impl Default for Mig2Config {
    fn default() -> Self {
        Self { evaluations: 200 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bayes1Config {
    pub evaluations: usize,
    pub initial_points: usize,
}

impl Default for Bayes1Config {
    fn default() -> Self {
        Self {
            evaluations: 200,
            initial_points: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LpminConfig {
    pub analysis_evals: usize,
    pub search_evals: usize,
    pub variable_order: Option<Vec<usize>>,
}

impl Default for LpminConfig {
    fn default() -> Self {
        Self {
            analysis_evals: 0,
            search_evals: 200,
            variable_order: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GloptConfig {
    pub evaluations: usize,
    pub initial_points: usize,
    pub local_trials: usize,
    pub shrink: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Mig1Config {
    pub evaluations: usize,
}

impl Default for Mig1Config {
    fn default() -> Self {
        Self { evaluations: 200 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UntConfig {
    pub evaluations: usize,
    pub local_step: f64,
}

impl Default for UntConfig {
    fn default() -> Self {
        Self {
            evaluations: 300,
            local_step: 0.15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExkorConfig {
    pub iterations: usize,
    pub step: f64,
    pub shrink: f64,
}

impl Default for ExkorConfig {
    fn default() -> Self {
        Self {
            iterations: 120,
            step: 0.25,
            shrink: 0.8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExtrConfig {
    pub evaluations: usize,
}

impl Default for ExtrConfig {
    fn default() -> Self {
        Self { evaluations: 240 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mivar4Config {
    pub iterations: usize,
    pub step: f64,
}

impl Default for Mivar4Config {
    fn default() -> Self {
        Self {
            iterations: 120,
            step: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexiConfig {
    pub iterations: usize,
    pub simplex_scale: f64,
}

impl Default for FlexiConfig {
    fn default() -> Self {
        Self {
            iterations: 180,
            simplex_scale: 0.08,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReqpConfig {
    pub iterations: usize,
    pub penalty: f64,
    pub penalty_growth: f64,
}

impl Default for ReqpConfig {
    fn default() -> Self {
        Self {
            iterations: 120,
            penalty: 10.0,
            penalty_growth: 1.25,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LbayesConfig {
    pub evaluations: usize,
    pub initial_points: usize,
    pub local_iterations: usize,
}

impl Default for LbayesConfig {
    fn default() -> Self {
        Self {
            evaluations: 220,
            initial_points: 30,
            local_iterations: 80,
        }
    }
}

impl Default for GloptConfig {
    fn default() -> Self {
        Self {
            evaluations: 400,
            initial_points: 80,
            local_trials: 5,
            shrink: 0.92,
        }
    }
}

pub fn mig2<F>(a: &[f64], b: &[f64], cfg: Mig2Config, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }

    let n = a.len();
    let mut ats = AtsGenerator::default();
    let mut points = Vec::with_capacity(cfg.evaluations);
    let mut values = Vec::with_capacity(cfg.evaluations);

    let mut best_f = f64::INFINITY;
    let mut best_x = vec![0.0; n];
    let mut best_iter = 0;

    let mut generated = Vec::with_capacity(cfg.evaluations);
    for _ in 1..=cfg.evaluations {
        generated.push(sample_with_ats(&mut ats, a, b));
    }

    let mut scored: Vec<(usize, f64, Vec<f64>)> = generated
        .into_iter()
        .enumerate()
        .map(|(idx, z)| (idx, f(&z), z))
        .collect();
    scored.sort_by_key(|(idx, _, _)| *idx);

    for (idx, ff, z) in scored {
        let iter = idx + 1;
        points.push(z.clone());
        values.push(ff);

        if ff < best_f {
            best_f = ff;
            best_x = z;
            best_iter = iter;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: cfg.evaluations,
        best_iter,
        points,
        values,
    })
}

pub fn bayes1<F>(a: &[f64], b: &[f64], cfg: Bayes1Config, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }
    if cfg.initial_points == 0 || cfg.initial_points > cfg.evaluations {
        return Err(OptError::InvalidInput(
            "initial_points must be in 1..=evaluations".to_string(),
        ));
    }

    let n = a.len();
    let mut points = Vec::with_capacity(cfg.evaluations);
    let mut values = Vec::with_capacity(cfg.evaluations);

    let mut best_f = f64::INFINITY;
    let mut best_x = vec![0.0; n];
    let mut best_iter = 0;

    let mut planner = BayesPlanner::new();

    for k in 1..=cfg.evaluations {
        let z = if k <= cfg.initial_points {
            let unit = lp_tau_point(k, n)?;
            map_unit_to_bounds(&unit, a, b)
        } else {
            planner.next_candidate(a, b, &points, &values, best_f)
        };

        let ff = f(&z);
        points.push(z.clone());
        values.push(ff);

        if ff < best_f {
            best_f = ff;
            best_x = z;
            best_iter = k;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: cfg.evaluations,
        best_iter,
        points,
        values,
    })
}

pub fn lpmin<F>(a: &[f64], b: &[f64], cfg: LpminConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.search_evals == 0 {
        return Err(OptError::InvalidInput(
            "search_evals must be > 0".to_string(),
        ));
    }

    let n = a.len();
    let order = resolve_variable_order(n, cfg.variable_order.as_deref())?;

    let mut points = Vec::with_capacity(cfg.analysis_evals + cfg.search_evals);
    let mut values = Vec::with_capacity(cfg.analysis_evals + cfg.search_evals);

    let mut best_f = f64::INFINITY;
    let mut best_x = vec![0.0; n];
    let mut best_iter = 0;

    let mut learned_order = order.clone();
    if cfg.analysis_evals > 0 && cfg.variable_order.is_none() {
        let mut analysis_points = Vec::with_capacity(cfg.analysis_evals);
        let mut analysis_values = Vec::with_capacity(cfg.analysis_evals);

        let mut analysis_generated = Vec::with_capacity(cfg.analysis_evals);
        for k in 1..=cfg.analysis_evals {
            let unit = lp_tau_point(k, n)?;
            analysis_generated.push(map_unit_to_bounds(&unit, a, b));
        }
        let mut analysis_scored: Vec<(usize, f64, Vec<f64>)> = analysis_generated
            .into_iter()
            .enumerate()
            .map(|(idx, z)| (idx, f(&z), z))
            .collect();
        analysis_scored.sort_by_key(|(idx, _, _)| *idx);

        for (idx, ff, z) in analysis_scored {
            points.push(z.clone());
            values.push(ff);
            analysis_points.push(z.clone());
            analysis_values.push(ff);

            if ff < best_f {
                best_f = ff;
                best_x = z;
                best_iter = idx + 1;
            }
        }

        learned_order = infer_variable_order(&analysis_points, &analysis_values);
    }

    let mut search_generated = Vec::with_capacity(cfg.search_evals);
    for k in 1..=cfg.search_evals {
        let unit = lp_tau_point(k, n)?;
        let mut z = vec![0.0; n];
        for j in 0..n {
            let idx = learned_order[j];
            z[idx] = a[idx] + unit[j] * (b[idx] - a[idx]);
        }
        search_generated.push(z);
    }
    let mut search_scored: Vec<(usize, f64, Vec<f64>)> = search_generated
        .into_iter()
        .enumerate()
        .map(|(idx, z)| (idx, f(&z), z))
        .collect();
    search_scored.sort_by_key(|(idx, _, _)| *idx);

    for (idx, ff, z) in search_scored {
        points.push(z.clone());
        values.push(ff);

        let iter = cfg.analysis_evals + idx + 1;
        if ff < best_f {
            best_f = ff;
            best_x = z;
            best_iter = iter;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: cfg.analysis_evals + cfg.search_evals,
        best_iter,
        points,
        values,
    })
}

pub fn glopt<F>(a: &[f64], b: &[f64], cfg: GloptConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }
    if cfg.initial_points == 0 || cfg.initial_points > cfg.evaluations {
        return Err(OptError::InvalidInput(
            "initial_points must be in 1..=evaluations".to_string(),
        ));
    }
    if cfg.local_trials == 0 {
        return Err(OptError::InvalidInput(
            "local_trials must be > 0".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.shrink) || cfg.shrink == 0.0 {
        return Err(OptError::InvalidInput(
            "shrink must be in (0, 1]".to_string(),
        ));
    }

    let n = a.len();
    let mut points = Vec::with_capacity(cfg.evaluations);
    let mut values = Vec::with_capacity(cfg.evaluations);

    let mut best_f = f64::INFINITY;
    let mut best_x = vec![0.0; n];
    let mut best_iter = 0;

    for k in 1..=cfg.initial_points {
        let unit = lp_tau_point(k, n)?;
        let z = map_unit_to_bounds(&unit, a, b);
        let ff = f(&z);

        points.push(z.clone());
        values.push(ff);

        if ff < best_f {
            best_f = ff;
            best_x = z;
            best_iter = k;
        }
    }

    let mut active_centers = select_cluster_centers(a, b, &points, &values);
    if active_centers.is_empty() {
        active_centers.push((best_x.clone(), best_f));
    }

    let mut ats = AtsGenerator::default();
    let mut step = 0.25_f64;
    let mut evals = cfg.initial_points;
    let mut center_idx = 0_usize;

    while evals < cfg.evaluations {
        let (center, center_val) = active_centers[center_idx].clone();
        let mut local_best_x = center;
        let mut local_best_f = center_val;

        for _ in 0..cfg.local_trials {
            if evals >= cfg.evaluations {
                break;
            }
            let mut cand = local_best_x.clone();
            for i in 0..n {
                let width = b[i] - a[i];
                let delta = (2.0 * ats.next() - 1.0) * step * width;
                cand[i] = clamp(cand[i] + delta, a[i], b[i]);
            }

            let ff = f(&cand);
            evals += 1;
            points.push(cand.clone());
            values.push(ff);

            if ff < local_best_f {
                local_best_f = ff;
                local_best_x = cand.clone();
            }
            if ff < best_f {
                best_f = ff;
                best_x = cand;
                best_iter = evals;
            }
        }

        active_centers[center_idx] = (local_best_x, local_best_f);
        center_idx = (center_idx + 1) % active_centers.len();
        step = (step * cfg.shrink).max(1.0e-5);
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals,
        best_iter,
        points,
        values,
    })
}

pub fn mig1<F>(a: &[f64], b: &[f64], cfg: Mig1Config, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }

    let n = a.len();
    let mut points = Vec::with_capacity(cfg.evaluations);
    let mut values = Vec::with_capacity(cfg.evaluations);

    let mut best_f = f64::INFINITY;
    let mut best_x = vec![0.0; n];
    let mut best_iter = 0;

    let mut generated = Vec::with_capacity(cfg.evaluations);
    for k in 1..=cfg.evaluations {
        let unit = lp_tau_point(k, n)?;
        generated.push(map_unit_to_bounds(&unit, a, b));
    }
    let mut scored: Vec<(usize, f64, Vec<f64>)> = generated
        .into_iter()
        .enumerate()
        .map(|(idx, z)| (idx, f(&z), z))
        .collect();
    scored.sort_by_key(|(idx, _, _)| *idx);

    for (idx, ff, z) in scored {
        points.push(z.clone());
        values.push(ff);

        if ff < best_f {
            best_f = ff;
            best_x = z;
            best_iter = idx + 1;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: cfg.evaluations,
        best_iter,
        points,
        values,
    })
}

pub fn unt<F>(a: &[f64], b: &[f64], cfg: UntConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if cfg.evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }

    let n = a.len();
    let mut ats = AtsGenerator::default();
    let mut points = Vec::with_capacity(cfg.evaluations);
    let mut values = Vec::with_capacity(cfg.evaluations);

    let mut best_x = map_unit_to_bounds(&lp_tau_point(1, n)?, a, b);
    let mut best_f = f(&best_x);
    let mut best_iter = 1;
    points.push(best_x.clone());
    values.push(best_f);

    for k in 2..=cfg.evaluations {
        let mut cand = map_unit_to_bounds(&lp_tau_point(k, n)?, a, b);
        if k % 2 == 0 {
            for i in 0..n {
                let delta = (2.0 * ats.next() - 1.0) * cfg.local_step * (b[i] - a[i]);
                cand[i] = clamp(best_x[i] + delta, a[i], b[i]);
            }
        }

        let ff = f(&cand);
        points.push(cand.clone());
        values.push(ff);

        if ff < best_f {
            best_f = ff;
            best_x = cand;
            best_iter = k;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: cfg.evaluations,
        best_iter,
        points,
        values,
    })
}

pub fn exkor<F>(x0: &[f64], a: &[f64], b: &[f64], cfg: ExkorConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if x0.len() != a.len() {
        return Err(OptError::InvalidInput(
            "x0 length must match bounds".to_string(),
        ));
    }

    let n = a.len();
    let mut x = x0
        .iter()
        .enumerate()
        .map(|(i, &v)| clamp(v, a[i], b[i]))
        .collect::<Vec<_>>();
    let mut step = cfg.step.max(1.0e-6);

    let mut points = Vec::with_capacity(cfg.iterations + 1);
    let mut values = Vec::with_capacity(cfg.iterations + 1);

    let mut best_f = f(&x);
    let mut best_x = x.clone();
    points.push(x.clone());
    values.push(best_f);
    let mut best_iter = 1;
    let mut evals = 1;

    for it in 1..=cfg.iterations {
        let mut improved = false;
        for i in 0..n {
            let width = b[i] - a[i];
            let h = step * width;
            let mut x_plus = x.clone();
            x_plus[i] = clamp(x_plus[i] + h, a[i], b[i]);
            let f_plus = f(&x_plus);
            evals += 1;
            points.push(x_plus.clone());
            values.push(f_plus);

            if f_plus < best_f {
                best_f = f_plus;
                best_x = x_plus.clone();
                x = x_plus;
                best_iter = evals;
                improved = true;
                continue;
            }

            let mut x_minus = x.clone();
            x_minus[i] = clamp(x_minus[i] - h, a[i], b[i]);
            let f_minus = f(&x_minus);
            evals += 1;
            points.push(x_minus.clone());
            values.push(f_minus);

            if f_minus < best_f {
                best_f = f_minus;
                best_x = x_minus.clone();
                x = x_minus;
                best_iter = evals;
                improved = true;
            }
        }

        if !improved {
            step = (step * cfg.shrink).max(1.0e-6);
        }
        if step <= 1.0e-6 {
            let _ = it;
            break;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals,
        best_iter,
        points,
        values,
    })
}

pub fn extr<F>(x0: &[f64], a: &[f64], b: &[f64], cfg: ExtrConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    let local = exkor(
        x0,
        a,
        b,
        ExkorConfig {
            iterations: cfg.evaluations / 2,
            step: 0.35,
            shrink: 0.7,
        },
        &f,
    )?;

    let mut gl = mig1(
        a,
        b,
        Mig1Config {
            evaluations: cfg.evaluations.saturating_sub(local.evals).max(1),
        },
        &f,
    )?;

    if local.best_f < gl.best_f {
        gl.best_x = local.best_x;
        gl.best_f = local.best_f;
        gl.best_iter = local.best_iter;
    }
    gl.evals += local.evals;
    gl.points.extend(local.points);
    gl.values.extend(local.values);
    Ok(gl)
}

pub fn mivar4<F>(x0: &[f64], a: &[f64], b: &[f64], cfg: Mivar4Config, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if x0.len() != a.len() {
        return Err(OptError::InvalidInput(
            "x0 length must match bounds".to_string(),
        ));
    }

    let n = a.len();
    let mut x = x0
        .iter()
        .enumerate()
        .map(|(i, &v)| clamp(v, a[i], b[i]))
        .collect::<Vec<_>>();

    let mut h_inv = identity_matrix(n);
    let mut points = Vec::with_capacity(cfg.iterations + 1);
    let mut values = Vec::with_capacity(cfg.iterations + 1);

    let mut fx = f(&x);
    let mut evals = 1;
    points.push(x.clone());
    values.push(fx);

    let mut best_x = x.clone();
    let mut best_f = fx;
    let mut best_iter = 1;

    for _ in 0..cfg.iterations {
        let g = finite_diff_grad(&x, a, b, &f);
        evals += n;

        let mut dir = mat_vec_mul(&h_inv, &g);
        for v in &mut dir {
            *v = -*v;
        }

        let mut alpha = cfg.step;
        let mut accepted = false;
        let mut x_new = x.clone();
        let mut f_new = fx;
        for _ in 0..12 {
            for i in 0..n {
                x_new[i] = clamp(x[i] + alpha * dir[i], a[i], b[i]);
            }
            f_new = f(&x_new);
            evals += 1;
            if f_new < fx {
                accepted = true;
                break;
            }
            alpha *= 0.5;
        }

        points.push(x_new.clone());
        values.push(f_new);

        if !accepted {
            continue;
        }

        let g_new = finite_diff_grad(&x_new, a, b, &f);
        evals += n;
        let s = vec_sub(&x_new, &x);
        let y = vec_sub(&g_new, &g);
        h_inv = bfgs_update_inverse(h_inv, &s, &y);

        x = x_new;
        fx = f_new;

        if fx < best_f {
            best_f = fx;
            best_x = x.clone();
            best_iter = evals;
        }
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals,
        best_iter,
        points,
        values,
    })
}

pub fn flexi<F>(x0: &[f64], a: &[f64], b: &[f64], cfg: FlexiConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if x0.len() != a.len() {
        return Err(OptError::InvalidInput(
            "x0 length must match bounds".to_string(),
        ));
    }

    let n = a.len();
    let mut simplex = Vec::with_capacity(n + 1);
    let x_start = x0
        .iter()
        .enumerate()
        .map(|(i, &v)| clamp(v, a[i], b[i]))
        .collect::<Vec<_>>();
    simplex.push(x_start.clone());
    for i in 0..n {
        let mut p = x_start.clone();
        p[i] = clamp(p[i] + cfg.simplex_scale * (b[i] - a[i]), a[i], b[i]);
        simplex.push(p);
    }

    let mut fvals: Vec<f64> = simplex.iter().map(|x| f(x)).collect();
    let mut evals = simplex.len();
    let mut points = simplex.clone();
    let mut values = fvals.clone();

    for _ in 0..cfg.iterations {
        let mut idx: Vec<usize> = (0..simplex.len()).collect();
        idx.sort_by(|&i, &j| fvals[i].partial_cmp(&fvals[j]).unwrap_or(std::cmp::Ordering::Equal));

        let best = idx[0];
        let worst = idx[n];
        let second_worst = idx[n - 1];

        let centroid = centroid_excluding(&simplex, worst);
        let xr = reflect(&centroid, &simplex[worst], 1.0);
        let xr = project_to_bounds(&xr, a, b);
        let fr = f(&xr);
        evals += 1;

        if fr < fvals[best] {
            let xe = reflect(&centroid, &simplex[worst], 2.0);
            let xe = project_to_bounds(&xe, a, b);
            let fe = f(&xe);
            evals += 1;
            if fe < fr {
                simplex[worst] = xe.clone();
                fvals[worst] = fe;
                points.push(xe);
                values.push(fe);
            } else {
                simplex[worst] = xr.clone();
                fvals[worst] = fr;
                points.push(xr);
                values.push(fr);
            }
            continue;
        }

        if fr < fvals[second_worst] {
            simplex[worst] = xr.clone();
            fvals[worst] = fr;
            points.push(xr);
            values.push(fr);
            continue;
        }

        let xc = reflect(&centroid, &simplex[worst], 0.5);
        let xc = project_to_bounds(&xc, a, b);
        let fc = f(&xc);
        evals += 1;
        if fc < fvals[worst] {
            simplex[worst] = xc.clone();
            fvals[worst] = fc;
            points.push(xc);
            values.push(fc);
            continue;
        }

        for i in 0..simplex.len() {
            if i == best {
                continue;
            }
            for d in 0..n {
                simplex[i][d] = 0.5 * (simplex[i][d] + simplex[best][d]);
                simplex[i][d] = clamp(simplex[i][d], a[d], b[d]);
            }
            fvals[i] = f(&simplex[i]);
            evals += 1;
            points.push(simplex[i].clone());
            values.push(fvals[i]);
        }
    }

    let mut best = 0usize;
    for i in 1..fvals.len() {
        if fvals[i] < fvals[best] {
            best = i;
        }
    }

    Ok(OptResult {
        best_x: simplex[best].clone(),
        best_f: fvals[best],
        evals,
        best_iter: evals,
        points,
        values,
    })
}

pub fn reqp<F, G>(
    x0: &[f64],
    a: &[f64],
    b: &[f64],
    cfg: ReqpConfig,
    f: F,
    constr: G,
) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64]) -> Vec<f64>,
{
    validate_bounds(a, b)?;
    if x0.len() != a.len() {
        return Err(OptError::InvalidInput(
            "x0 length must match bounds".to_string(),
        ));
    }

    let mut mu = cfg.penalty.max(1.0e-6);
    let mut current_x = x0.to_vec();
    let mut all_points = Vec::new();
    let mut all_values = Vec::new();
    let mut total_evals = 0;

    let mut best_x = current_x.clone();
    let mut best_f = penalized_obj(&current_x, mu, &f, &constr);
    let mut best_iter = 1;

    for _ in 0..cfg.iterations {
        let res = mivar4(
            &current_x,
            a,
            b,
            Mivar4Config {
                iterations: 1,
                step: 0.15,
            },
            |x| penalized_obj(x, mu, &f, &constr),
        )?;
        total_evals += res.evals;
        all_points.extend(res.points.clone());
        all_values.extend(res.values.clone());
        current_x = res.best_x;

        let base = f(&current_x);
        if base < best_f {
            best_f = base;
            best_x = current_x.clone();
            best_iter = total_evals;
        }
        mu *= cfg.penalty_growth.max(1.0);
    }

    Ok(OptResult {
        best_x,
        best_f,
        evals: total_evals,
        best_iter,
        points: all_points,
        values: all_values,
    })
}

pub fn lbayes<F>(a: &[f64], b: &[f64], cfg: LbayesConfig, f: F) -> Result<OptResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    let coarse = bayes1(
        a,
        b,
        Bayes1Config {
            evaluations: cfg.evaluations,
            initial_points: cfg.initial_points,
        },
        &f,
    )?;

    let local = flexi(
        &coarse.best_x,
        a,
        b,
        FlexiConfig {
            iterations: cfg.local_iterations,
            simplex_scale: 0.06,
        },
        &f,
    )?;

    let mut result = coarse.clone();
    if local.best_f < result.best_f {
        result.best_x = local.best_x;
        result.best_f = local.best_f;
    }
    result.evals += local.evals;
    result.points.extend(local.points);
    result.values.extend(local.values);
    result.best_iter = result.evals;
    Ok(result)
}

pub fn anal1<F>(a: &[f64], b: &[f64], samples: usize, f: F) -> Result<AnalResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if samples == 0 {
        return Err(OptError::InvalidInput("samples must be > 0".to_string()));
    }
    let n = a.len();
    let mut pts = Vec::with_capacity(samples);
    let mut vals = Vec::with_capacity(samples);
    let mut generated = Vec::with_capacity(samples);
    for k in 1..=samples {
        generated.push(map_unit_to_bounds(&lp_tau_point(k, n)?, a, b));
    }
    let mut scored: Vec<(usize, f64, Vec<f64>)> = generated
        .into_iter()
        .enumerate()
        .map(|(idx, p)| (idx, f(&p), p))
        .collect();
    scored.sort_by_key(|(idx, _, _)| *idx);
    for (_, val, p) in scored {
        vals.push(val);
        pts.push(p);
    }

    let order = infer_variable_order(&pts, &vals);
    let scores = influence_scores(&pts, &vals);
    Ok(AnalResult {
        variable_order: order,
        influence_scores: scores,
        samples,
    })
}

pub fn anal2<F>(a: &[f64], b: &[f64], samples: usize, f: F) -> Result<AnalResult, OptError>
where
    F: Fn(&[f64]) -> f64,
{
    validate_bounds(a, b)?;
    if samples == 0 {
        return Err(OptError::InvalidInput("samples must be > 0".to_string()));
    }
    let n = a.len();
    let mut pts = Vec::with_capacity(samples);
    let mut vals = Vec::with_capacity(samples);
    let mut generated = Vec::with_capacity(samples);
    for k in 1..=samples {
        let mut p = map_unit_to_bounds(&lp_tau_point(k, n)?, a, b);
        for i in 0..n {
            p[i] = a[i] + (b[i] - a[i]) * (p[i] * p[i]);
        }
        generated.push(p);
    }
    let mut scored: Vec<(usize, f64, Vec<f64>)> = generated
        .into_iter()
        .enumerate()
        .map(|(idx, p)| (idx, f(&p), p))
        .collect();
    scored.sort_by_key(|(idx, _, _)| *idx);
    for (_, val, p) in scored {
        vals.push(val);
        pts.push(p);
    }

    let mut scores = influence_scores(&pts, &vals);
    for s in &mut scores {
        *s = s.sqrt();
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| {
        scores[j]
            .partial_cmp(&scores[i])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(AnalResult {
        variable_order: order,
        influence_scores: scores,
        samples,
    })
}

fn validate_bounds(a: &[f64], b: &[f64]) -> Result<(), OptError> {
    if a.is_empty() || b.is_empty() {
        return Err(OptError::InvalidInput(
            "bounds must not be empty".to_string(),
        ));
    }
    if a.len() != b.len() {
        return Err(OptError::InvalidInput(
            "bounds must have the same length".to_string(),
        ));
    }
    if a.len() > 20 {
        return Err(OptError::InvalidInput(
            "dimension must be <= 20 (as in original Fortran code)".to_string(),
        ));
    }
    for i in 0..a.len() {
        if a[i] > b[i] {
            return Err(OptError::InvalidInput(format!(
                "invalid interval at index {i}: a > b"
            )));
        }
    }
    Ok(())
}

fn resolve_variable_order(n: usize, order: Option<&[usize]>) -> Result<Vec<usize>, OptError> {
    if let Some(ord) = order {
        if ord.len() != n {
            return Err(OptError::InvalidInput(
                "variable_order length must match dimension".to_string(),
            ));
        }
        let mut seen = vec![false; n];
        for &idx in ord {
            if idx >= n {
                return Err(OptError::InvalidInput(
                    "variable_order contains out-of-range index".to_string(),
                ));
            }
            if seen[idx] {
                return Err(OptError::InvalidInput(
                    "variable_order must be a permutation".to_string(),
                ));
            }
            seen[idx] = true;
        }
        return Ok(ord.to_vec());
    }
    Ok((0..n).collect())
}

fn infer_variable_order(points: &[Vec<f64>], values: &[f64]) -> Vec<usize> {
    let n = points[0].len();
    let m = points.len();
    let y_mean = values.iter().copied().sum::<f64>() / m as f64;

    let mut scores = vec![0.0; n];
    for i in 0..n {
        let x_mean = points.iter().map(|p| p[i]).sum::<f64>() / m as f64;
        let mut cov = 0.0;
        let mut var = 0.0;
        for (p, &y) in points.iter().zip(values.iter()) {
            let dx = p[i] - x_mean;
            let dy = y - y_mean;
            cov += dx * dy;
            var += dx * dx;
        }
        scores[i] = if var > 0.0 { (cov / var).abs() } else { 0.0 };
    }

    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| {
        scores[j]
            .partial_cmp(&scores[i])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    idx
}

fn select_cluster_centers(
    a: &[f64],
    b: &[f64],
    points: &[Vec<f64>],
    values: &[f64],
) -> Vec<(Vec<f64>, f64)> {
    let n = a.len();
    let mut diag_sq = 0.0;
    for i in 0..n {
        let w = b[i] - a[i];
        diag_sq += w * w;
    }
    let diag = diag_sq.sqrt();
    let scale = (points.len() as f64).powf(1.0 / n as f64);
    let radius = (diag / scale) * 0.2;

    let mut order: Vec<usize> = (0..points.len()).collect();
    order.sort_by(|&i, &j| {
        values[i]
            .partial_cmp(&values[j])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut selected: Vec<(Vec<f64>, f64)> = Vec::new();
    'outer: for &idx in &order {
        let p = &points[idx];
        for (c, _) in &selected {
            let d = euclidean_distance(p, c);
            if d < radius {
                continue 'outer;
            }
        }
        selected.push((p.clone(), values[idx]));
        if selected.len() >= 12 {
            break;
        }
    }
    selected
}

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f64>()
        .sqrt()
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

fn influence_scores(points: &[Vec<f64>], values: &[f64]) -> Vec<f64> {
    let n = points[0].len();
    let m = points.len();
    let y_mean = values.iter().copied().sum::<f64>() / m as f64;
    let y_var = values
        .iter()
        .map(|&y| {
            let d = y - y_mean;
            d * d
        })
        .sum::<f64>()
        .max(1.0e-12);

    let mut scores = vec![0.0; n];
    for i in 0..n {
        let x_mean = points.iter().map(|p| p[i]).sum::<f64>() / m as f64;
        let mut cov = 0.0;
        let mut x_var = 0.0;
        for (p, &y) in points.iter().zip(values.iter()) {
            let dx = p[i] - x_mean;
            let dy = y - y_mean;
            cov += dx * dy;
            x_var += dx * dx;
        }
        scores[i] = if x_var > 0.0 {
            (cov * cov) / (x_var * y_var)
        } else {
            0.0
        };
    }
    scores
}

fn identity_matrix(n: usize) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; n]; n];
    for (i, row) in m.iter_mut().enumerate().take(n) {
        row[i] = 1.0;
    }
    m
}

fn finite_diff_grad<F>(x: &[f64], a: &[f64], b: &[f64], f: &F) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    let fx = f(x);
    let mut g = vec![0.0; n];
    for i in 0..n {
        let h = 1.0e-6_f64.max(1.0e-4 * (b[i] - a[i]));
        let mut xp = x.to_vec();
        xp[i] = clamp(xp[i] + h, a[i], b[i]);
        let fp = f(&xp);
        g[i] = (fp - fx) / (xp[i] - x[i]).max(1.0e-12);
    }
    g
}

fn mat_vec_mul(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|row| row.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

fn vec_sub(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn bfgs_update_inverse(mut h: Vec<Vec<f64>>, s: &[f64], y: &[f64]) -> Vec<Vec<f64>> {
    let sty = dot(s, y);
    if sty.abs() < 1.0e-12 {
        return h;
    }
    let rho = 1.0 / sty;
    let hy = mat_vec_mul(&h, y);
    let yhy = dot(y, &hy);
    let n = s.len();

    for i in 0..n {
        for j in 0..n {
            h[i][j] += (1.0 + yhy * rho) * rho * s[i] * s[j] - rho * (s[i] * hy[j] + hy[i] * s[j]);
        }
    }
    h
}

fn centroid_excluding(simplex: &[Vec<f64>], exclude: usize) -> Vec<f64> {
    let n = simplex[0].len();
    let mut c = vec![0.0; n];
    let mut cnt = 0.0;
    for (i, p) in simplex.iter().enumerate() {
        if i == exclude {
            continue;
        }
        for d in 0..n {
            c[d] += p[d];
        }
        cnt += 1.0;
    }
    for v in &mut c {
        *v /= cnt;
    }
    c
}

fn reflect(c: &[f64], w: &[f64], alpha: f64) -> Vec<f64> {
    c.iter()
        .zip(w.iter())
        .map(|(ci, wi)| ci + alpha * (ci - wi))
        .collect()
}

fn project_to_bounds(x: &[f64], a: &[f64], b: &[f64]) -> Vec<f64> {
    x.iter()
        .enumerate()
        .map(|(i, &v)| clamp(v, a[i], b[i]))
        .collect()
}

fn penalized_obj<F, G>(x: &[f64], mu: f64, f: &F, constr: &G) -> f64
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64]) -> Vec<f64>,
{
    let mut p = 0.0;
    for g in constr(x) {
        let viol = g.max(0.0);
        p += viol * viol;
    }
    f(x) + mu * p
}

fn map_unit_to_bounds(unit: &[f64], a: &[f64], b: &[f64]) -> Vec<f64> {
    unit.iter()
        .enumerate()
        .map(|(i, &u)| a[i] + u * (b[i] - a[i]))
        .collect()
}

fn sample_with_ats(ats: &mut AtsGenerator, a: &[f64], b: &[f64]) -> Vec<f64> {
    (0..a.len())
        .map(|i| a[i] + ats.next() * (b[i] - a[i]))
        .collect()
}

struct BayesPlanner {
    ats: AtsGenerator,
    x2: Option<Vec<f64>>,
}

impl BayesPlanner {
    fn new() -> Self {
        Self {
            ats: AtsGenerator::default(),
            x2: None,
        }
    }

    // Streaming two-best candidate scan, mirroring the upstream MIG2F2:
    // candidates are scored one at a time while FMIN (the running
    // second-best score) tightens the FIAP1 early-exit bound.
    fn next_candidate(
        &mut self,
        a: &[f64],
        b: &[f64],
        points: &[Vec<f64>],
        values: &[f64],
        ym: f64,
    ) -> Vec<f64> {
        let n = a.len();
        let l = points.len();
        let m = (50 * n).min((10 * n).max(l * n)).max(2);

        let mut fmin = 0.0; // COMMON /BAYFM/: second-best score bound
        // Upstream: a fresh X2 consumes one of the M candidate draws.
        let (mut x2, m) = match self.x2.take() {
            Some(x2) => (x2, m),
            None => (sample_with_ats(&mut self.ats, a, b), m - 1),
        };
        let mut f2 = fiap1(&x2, points, values, ym, fmin);

        let mut best_x = x2.clone();
        let mut best_f = f64::INFINITY;

        for k in 0..m {
            let z = sample_with_ats(&mut self.ats, a, b);
            let ff = fiap1(&z, points, values, ym, fmin);

            if k == 0 {
                if ff < f2 {
                    best_x = z;
                    best_f = ff;
                    fmin = f2;
                } else {
                    best_x = x2.clone();
                    best_f = f2;
                    x2 = z;
                    f2 = ff;
                    fmin = ff;
                }
                continue;
            }
            if ff >= f2 {
                continue;
            }
            if ff >= best_f {
                x2 = z;
                f2 = ff;
                fmin = ff;
                continue;
            }
            f2 = best_f;
            x2 = std::mem::replace(&mut best_x, z);
            best_f = ff;
            fmin = f2;
        }

        self.x2 = Some(x2);
        best_x
    }
}

fn fiap1(x: &[f64], points: &[Vec<f64>], values: &[f64], ym: f64, fmin: f64) -> f64 {
    if points.is_empty() || values.is_empty() {
        return 0.0;
    }

    let rmax = f64::MAX / 2.0;
    let rlrs = 100.0 * f64::EPSILON * ym.abs();
    let e = 1.0e-6_f64.max(rlrs);
    let yk = ym - e;
    let fm = -fmin; // upstream FM = -FMIN early-termination bound

    let mut p = 1.0_f64.max(values[0] - yk);
    if p.abs() < 1.0e-12 {
        p = 1.0;
    }
    let mut fii = rmax / p;

    'scan: for (obs, &y) in points.iter().zip(values.iter()) {
        let pp = y - yk;
        if pp.abs() < 1.0e-12 {
            continue;
        }

        let threshold = if pp < 1.0 {
            pp * fii
        } else if rmax / pp > fii {
            pp * fii
        } else {
            rmax
        };

        // Accumulate the squared distance with the upstream early exit:
        // once the partial sum reaches the threshold this observation
        // cannot lower FII, so it is skipped (IF(D.GE.P) GOTO 20).
        let mut d = 0.0;
        for (xo, xi) in obs.iter().zip(x.iter()) {
            let diff = xo - xi;
            d += diff * diff;
            if d >= threshold {
                continue 'scan;
            }
        }
        fii = d / pp;
        // Upstream IF(FII.LE.FM) GOTO 30: the score already cannot beat
        // the current two best candidates, stop scanning.
        if fii <= fm {
            break;
        }
    }

    -fii
}
