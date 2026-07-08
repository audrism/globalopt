use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::optimizers::{
    anal1, anal2, bayes1, exkor, extr, flexi, glopt, lbayes, lpmin, mig1, mig2, mivar4, reqp,
    unt, AnalResult, Bayes1Config, ExkorConfig, ExtrConfig, FlexiConfig, GloptConfig,
    LbayesConfig, LpminConfig, Mig1Config, Mig2Config, Mivar4Config, OptResult, ReqpConfig,
    UntConfig,
};

#[pyfunction]
fn furasn_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::furasn(&x)
}

#[pyfunction]
fn fush5_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fush5(&x)
}

#[pyfunction]
fn fush7_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fush7(&x)
}

#[pyfunction]
fn fush10_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fush10(&x)
}

#[pyfunction]
fn fuhar3_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fuhar3(&x)
}

#[pyfunction]
fn fuhar6_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fuhar6(&x)
}

#[pyfunction]
fn fubran_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fubran(&x)
}

#[pyfunction]
fn fugold_py(x: Vec<f64>) -> f64 {
    crate::benchmarks::fugold(&x)
}

#[pyfunction]
fn lp_tau_point_py(c: usize, n: usize) -> PyResult<Vec<f64>> {
    crate::lptau::lp_tau_point(c, n).map_err(to_pyerr)
}

#[pyfunction]
fn mig1_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = mig1(&a, &b, Mig1Config { evaluations }, |x| eval_objective(py, &objective, x))
        .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn mig2_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = mig2(&a, &b, Mig2Config { evaluations }, |x| eval_objective(py, &objective, x))
        .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
#[pyo3(signature = (a, b, evaluations, initial_points, objective, local_minimizer=None))]
fn bayes1_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    initial_points: usize,
    objective: PyObject,
    local_minimizer: Option<PyObject>,
) -> PyResult<PyObject> {
    let mut result = bayes1(
        &a,
        &b,
        Bayes1Config {
            evaluations,
            initial_points,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;

    if let Some(local_minimizer) = local_minimizer {
        if let Some((par, value)) = apply_local_minimizer(py, &local_minimizer, &objective, &result.best_x, &a, &b)? {
            if value < result.best_f {
                result.best_x = par;
                result.best_f = value;
            }
        }
    }

    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn lpmin_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, analysis_evals: usize, search_evals: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = lpmin(
        &a,
        &b,
        LpminConfig {
            analysis_evals,
            search_evals,
            variable_order: None,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn glopt_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, initial_points: usize, local_trials: usize, shrink: f64, objective: PyObject) -> PyResult<PyObject> {
    let result = glopt(
        &a,
        &b,
        GloptConfig {
            evaluations,
            initial_points,
            local_trials,
            shrink,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn unt_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, local_step: f64, objective: PyObject) -> PyResult<PyObject> {
    let result = unt(
        &a,
        &b,
        UntConfig {
            evaluations,
            local_step,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn exkor_py(py: Python<'_>, x0: Vec<f64>, a: Vec<f64>, b: Vec<f64>, iterations: usize, step: f64, shrink: f64, objective: PyObject) -> PyResult<PyObject> {
    let result = exkor(
        &x0,
        &a,
        &b,
        ExkorConfig {
            iterations,
            step,
            shrink,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn extr_py(py: Python<'_>, x0: Vec<f64>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = extr(
        &x0,
        &a,
        &b,
        ExtrConfig { evaluations },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn mivar4_py(py: Python<'_>, x0: Vec<f64>, a: Vec<f64>, b: Vec<f64>, iterations: usize, step: f64, objective: PyObject) -> PyResult<PyObject> {
    let result = mivar4(
        &x0,
        &a,
        &b,
        Mivar4Config { iterations, step },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn flexi_py(py: Python<'_>, x0: Vec<f64>, a: Vec<f64>, b: Vec<f64>, iterations: usize, simplex_scale: f64, objective: PyObject) -> PyResult<PyObject> {
    let result = flexi(
        &x0,
        &a,
        &b,
        FlexiConfig {
            iterations,
            simplex_scale,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn reqp_py(py: Python<'_>, x0: Vec<f64>, a: Vec<f64>, b: Vec<f64>, iterations: usize, penalty: f64, penalty_growth: f64, objective: PyObject, constr: PyObject) -> PyResult<PyObject> {
    let result = reqp(
        &x0,
        &a,
        &b,
        ReqpConfig {
            iterations,
            penalty,
            penalty_growth,
        },
        |x| eval_objective(py, &objective, x),
        |x| eval_constraints(py, &constr, x),
    )
    .map_err(to_pyerr)?;
    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
#[pyo3(signature = (a, b, evaluations, initial_points, local_iterations, objective, local_minimizer=None))]
fn lbayes_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, evaluations: usize, initial_points: usize, local_iterations: usize, objective: PyObject, local_minimizer: Option<PyObject>) -> PyResult<PyObject> {
    let result = lbayes(
        &a,
        &b,
        LbayesConfig {
            evaluations,
            initial_points,
            local_iterations,
        },
        |x| eval_objective(py, &objective, x),
    )
    .map_err(to_pyerr)?;

    if let Some(local_minimizer) = local_minimizer {
        if let Some((par, value)) = apply_local_minimizer(py, &local_minimizer, &objective, &result.best_x, &a, &b)? {
            let mut refined = result.clone();
            if value < refined.best_f {
                refined.best_x = par;
                refined.best_f = value;
            }
            return Ok(opt_result_to_py(py, &refined));
        }
    }

    Ok(opt_result_to_py(py, &result))
}

#[pyfunction]
fn anal1_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, samples: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = anal1(&a, &b, samples, |x| eval_objective(py, &objective, x)).map_err(to_pyerr)?;
    Ok(anal_result_to_py(py, &result))
}

#[pyfunction]
fn anal2_py(py: Python<'_>, a: Vec<f64>, b: Vec<f64>, samples: usize, objective: PyObject) -> PyResult<PyObject> {
    let result = anal2(&a, &b, samples, |x| eval_objective(py, &objective, x)).map_err(to_pyerr)?;
    Ok(anal_result_to_py(py, &result))
}

fn eval_objective(py: Python<'_>, objective: &PyObject, x: &[f64]) -> f64 {
    let list = PyList::new_bound(py, x);
    objective.call1(py, (list,)).unwrap().extract(py).unwrap()
}

fn eval_constraints(py: Python<'_>, constr: &PyObject, x: &[f64]) -> Vec<f64> {
    let list = PyList::new_bound(py, x);
    constr.call1(py, (list,)).unwrap().extract(py).unwrap()
}

#[derive(FromPyObject)]
struct LocalMinimizerResult {
    par: Vec<f64>,
    value: f64,
}

fn apply_local_minimizer(
    py: Python<'_>,
    local_minimizer: &PyObject,
    objective: &PyObject,
    x0: &[f64],
    lower: &[f64],
    upper: &[f64],
) -> PyResult<Option<(Vec<f64>, f64)>> {
    let result = local_minimizer.call1(py, (
        objective,
        PyList::new_bound(py, x0),
        PyList::new_bound(py, lower),
        PyList::new_bound(py, upper),
    ))?;
    let parsed: LocalMinimizerResult = result.extract(py)?;
    Ok(Some((parsed.par, parsed.value)))
}

fn opt_result_to_py(py: Python<'_>, result: &OptResult) -> PyObject {
    let dict = PyDict::new_bound(py);
    dict.set_item("best_x", result.best_x.clone()).unwrap();
    dict.set_item("best_f", result.best_f).unwrap();
    dict.set_item("evals", result.evals).unwrap();
    dict.set_item("best_iter", result.best_iter).unwrap();
    dict.set_item("points", result.points.clone()).unwrap();
    dict.set_item("values", result.values.clone()).unwrap();
    dict.into_py(py)
}

fn anal_result_to_py(py: Python<'_>, result: &AnalResult) -> PyObject {
    let dict = PyDict::new_bound(py);
    dict.set_item("variable_order", result.variable_order.clone()).unwrap();
    dict.set_item("influence_scores", result.influence_scores.clone()).unwrap();
    dict.set_item("samples", result.samples).unwrap();
    dict.into_py(py)
}

fn to_pyerr(err: crate::optimizers::OptError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(err.to_string())
}

// ---------------------------------------------------------------------------
// Original-Fortran backend (feature "fortran"): each *_fortran_py function
// runs the genuine 1989 GlobalMinimum routine.  `objective` is either a
// Python callable or a builtin name ("furasn", "fush5", "fush7", "fush10",
// "fuhar3", "fuhar6", "fubran", "fugold"); `ats_state` optionally seeds the
// 15-word ATS random state (same seed => reproducible run).
// ---------------------------------------------------------------------------

#[cfg(feature = "fortran")]
mod fortran_support {
    use super::*;
    use crate::fortran::{builtin_index, FortranObjective};

    pub(super) enum FortranObjectiveSpec {
        Builtin(u32),
        Callable(PyObject),
    }

    pub(super) fn parse_fortran_objective(
        py: Python<'_>,
        objective: &PyObject,
    ) -> PyResult<FortranObjectiveSpec> {
        if let Ok(name) = objective.extract::<String>(py) {
            return builtin_index(&name)
                .map(FortranObjectiveSpec::Builtin)
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "unknown builtin objective '{name}' (expected one of furasn, fush5, \
                         fush7, fush10, fuhar3, fuhar6, fubran, fugold)"
                    ))
                });
        }
        if objective.bind(py).is_callable() {
            return Ok(FortranObjectiveSpec::Callable(objective.clone_ref(py)));
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "objective must be a callable or a builtin objective name",
        ))
    }

    pub(super) fn parse_ats_state(ats_state: Option<Vec<f64>>) -> PyResult<Option<[f64; 15]>> {
        match ats_state {
            None => Ok(None),
            Some(v) => {
                let arr: [f64; 15] = v.try_into().map_err(|_| {
                    pyo3::exceptions::PyValueError::new_err(
                        "ats_state must contain exactly 15 values",
                    )
                })?;
                Ok(Some(arr))
            }
        }
    }

    /// Runs `call` with the parsed objective, wrapping Python callables in
    /// a closure that evaluates them (GIL is held: the Fortran call happens
    /// on the calling Python thread).
    pub(super) fn with_fortran_objective<R>(
        py: Python<'_>,
        objective: &PyObject,
        call: impl FnOnce(FortranObjective<'_>) -> Result<R, crate::optimizers::OptError>,
    ) -> PyResult<R> {
        match parse_fortran_objective(py, objective)? {
            FortranObjectiveSpec::Builtin(idx) => {
                call(FortranObjective::Builtin(idx)).map_err(to_pyerr)
            }
            FortranObjectiveSpec::Callable(func) => {
                let mut closure = |x: &[f64]| eval_objective(py, &func, x);
                call(FortranObjective::Closure(&mut closure)).map_err(to_pyerr)
            }
        }
    }
}

#[cfg(feature = "fortran")]
use fortran_support::{parse_ats_state, with_fortran_objective};

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, evaluations, objective, ats_state=None))]
fn mig1_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_mig1(&a, &b, evaluations, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, evaluations, objective, ats_state=None))]
fn mig2_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_mig2(&a, &b, evaluations, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, evaluations, initial_points, objective, ats_state=None))]
fn bayes1_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    initial_points: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_bayes1(&a, &b, evaluations, initial_points, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, evaluations, initial_points, max_local_minima, objective, ats_state=None))]
fn unt_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    initial_points: usize,
    max_local_minima: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_unt(
            &a,
            &b,
            evaluations,
            initial_points,
            max_local_minima,
            obj,
            ats.as_ref(),
        )
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, evaluations, start_points, objective, ats_state=None))]
fn glopt_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    evaluations: usize,
    start_points: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_glopt(&a, &b, evaluations, start_points, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (a, b, analysis_evals, search_evals, objective, ats_state=None))]
fn lpmin_fortran_py(
    py: Python<'_>,
    a: Vec<f64>,
    b: Vec<f64>,
    analysis_evals: i32,
    search_evals: usize,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_lpmin(&a, &b, analysis_evals, search_evals, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (bp, ep, evaluations, model_evals, acc_y, acc_x, objective, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn extr_fortran_py(
    py: Python<'_>,
    bp: f64,
    ep: f64,
    evaluations: usize,
    model_evals: usize,
    acc_y: f64,
    acc_x: f64,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_extr(
            bp,
            ep,
            evaluations,
            model_evals,
            acc_y,
            acc_x,
            obj,
            ats.as_ref(),
        )
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (x0, a, b, evals_per_coord, model_evals, cycles, first_coord, acc, objective, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn exkor_fortran_py(
    py: Python<'_>,
    x0: Vec<f64>,
    a: Vec<f64>,
    b: Vec<f64>,
    evals_per_coord: usize,
    model_evals: usize,
    cycles: usize,
    first_coord: usize,
    acc: f64,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_exkor(
            &x0,
            &a,
            &b,
            evals_per_coord,
            model_evals,
            cycles,
            first_coord,
            acc,
            obj,
            ats.as_ref(),
        )
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (x0, a, b, max_evals, nstop, imax, xeps, eps, eps1, delta, objective, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn mivar4_fortran_py(
    py: Python<'_>,
    x0: Vec<f64>,
    a: Vec<f64>,
    b: Vec<f64>,
    max_evals: usize,
    nstop: usize,
    imax: usize,
    xeps: f64,
    eps: f64,
    eps1: f64,
    delta: f64,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_mivar4(
            &x0,
            &a,
            &b,
            max_evals,
            nstop,
            imax,
            xeps,
            eps,
            eps1,
            delta,
            obj,
            ats.as_ref(),
        )
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (x0, a, b, iterations, aniu, beta, objective, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn lbayes_fortran_py(
    py: Python<'_>,
    x0: Vec<f64>,
    a: Vec<f64>,
    b: Vec<f64>,
    iterations: usize,
    aniu: f64,
    beta: f64,
    objective: PyObject,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        crate::fortran::fortran_lbayes(&x0, &a, &b, iterations, aniu, beta, obj, ats.as_ref())
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (x0, max_evals, n_eq, n_ineq, size, conver, objective, constraints=None, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn flexi_fortran_py(
    py: Python<'_>,
    x0: Vec<f64>,
    max_evals: usize,
    n_eq: usize,
    n_ineq: usize,
    size: f64,
    conver: f64,
    objective: PyObject,
    constraints: Option<PyObject>,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        match &constraints {
            Some(con) => {
                let mut fill = |x: &[f64], r: &mut [f64]| {
                    let vals = eval_constraints(py, con, x);
                    for (ri, v) in r.iter_mut().zip(vals) {
                        *ri = v;
                    }
                };
                crate::fortran::fortran_flexi(
                    &x0,
                    max_evals,
                    n_eq,
                    n_ineq,
                    size,
                    conver,
                    obj,
                    Some(&mut fill),
                    ats.as_ref(),
                )
            }
            None => crate::fortran::fortran_flexi(
                &x0,
                max_evals,
                n_eq,
                n_ineq,
                size,
                conver,
                obj,
                None,
                ats.as_ref(),
            ),
        }
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
#[pyo3(signature = (x0, imax, n_eq, n_ineq, r1, scale, delta, eps, objective, constraints=None, ats_state=None))]
#[allow(clippy::too_many_arguments)]
fn reqp_fortran_py(
    py: Python<'_>,
    x0: Vec<f64>,
    imax: usize,
    n_eq: usize,
    n_ineq: usize,
    r1: f64,
    scale: f64,
    delta: f64,
    eps: f64,
    objective: PyObject,
    constraints: Option<PyObject>,
    ats_state: Option<Vec<f64>>,
) -> PyResult<PyObject> {
    let ats = parse_ats_state(ats_state)?;
    let result = with_fortran_objective(py, &objective, |obj| {
        match &constraints {
            Some(con) => {
                let mut fill = |x: &[f64], r: &mut [f64]| {
                    let vals = eval_constraints(py, con, x);
                    for (ri, v) in r.iter_mut().zip(vals) {
                        *ri = v;
                    }
                };
                crate::fortran::fortran_reqp(
                    &x0,
                    imax,
                    n_eq,
                    n_ineq,
                    r1,
                    scale,
                    delta,
                    eps,
                    obj,
                    Some(&mut fill),
                    ats.as_ref(),
                )
            }
            None => crate::fortran::fortran_reqp(
                &x0,
                imax,
                n_eq,
                n_ineq,
                r1,
                scale,
                delta,
                eps,
                obj,
                None,
                ats.as_ref(),
            ),
        }
    })?;
    Ok(opt_result_to_py(py, &result))
}

#[cfg(feature = "fortran")]
#[pyfunction]
fn lptau_fortran_py(c: usize, n: usize) -> PyResult<Vec<f64>> {
    crate::fortran::fortran_lptau(c, n).map_err(to_pyerr)
}

#[pymodule]
pub fn globalopt_native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(furasn_py, m)?)?;
    m.add_function(wrap_pyfunction!(fush5_py, m)?)?;
    m.add_function(wrap_pyfunction!(fush7_py, m)?)?;
    m.add_function(wrap_pyfunction!(fush10_py, m)?)?;
    m.add_function(wrap_pyfunction!(fuhar3_py, m)?)?;
    m.add_function(wrap_pyfunction!(fuhar6_py, m)?)?;
    m.add_function(wrap_pyfunction!(fubran_py, m)?)?;
    m.add_function(wrap_pyfunction!(fugold_py, m)?)?;
    m.add_function(wrap_pyfunction!(lp_tau_point_py, m)?)?;
    m.add_function(wrap_pyfunction!(mig1_py, m)?)?;
    m.add_function(wrap_pyfunction!(mig2_py, m)?)?;
    m.add_function(wrap_pyfunction!(bayes1_py, m)?)?;
    m.add_function(wrap_pyfunction!(lpmin_py, m)?)?;
    m.add_function(wrap_pyfunction!(glopt_py, m)?)?;
    m.add_function(wrap_pyfunction!(unt_py, m)?)?;
    m.add_function(wrap_pyfunction!(exkor_py, m)?)?;
    m.add_function(wrap_pyfunction!(extr_py, m)?)?;
    m.add_function(wrap_pyfunction!(mivar4_py, m)?)?;
    m.add_function(wrap_pyfunction!(flexi_py, m)?)?;
    m.add_function(wrap_pyfunction!(reqp_py, m)?)?;
    m.add_function(wrap_pyfunction!(lbayes_py, m)?)?;
    m.add_function(wrap_pyfunction!(anal1_py, m)?)?;
    m.add_function(wrap_pyfunction!(anal2_py, m)?)?;
    m.add("HAS_FORTRAN", cfg!(feature = "fortran"))?;
    #[cfg(feature = "fortran")]
    {
        m.add_function(wrap_pyfunction!(mig1_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(mig2_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(bayes1_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(unt_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(glopt_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(lpmin_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(extr_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(exkor_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(mivar4_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(lbayes_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(flexi_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(reqp_fortran_py, m)?)?;
        m.add_function(wrap_pyfunction!(lptau_fortran_py, m)?)?;
    }
    Ok(())
}