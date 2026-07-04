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
    Ok(())
}