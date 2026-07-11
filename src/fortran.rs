//! Safe wrappers around the original GlobalMinimum Fortran library
//! (Mockus, 1989), linked in by build.rs when the "fortran" feature is
//! enabled.
//!
//! The Fortran routines resolve the objective FI(X,N) at link time via
//! the trampoline in fortran/gm_fi.f, which forwards to the C shim
//! (fortran/gm_shim.c).  The shim dispatches each evaluation either to
//! a registered host callback or to a compiled built-in objective,
//! counts evaluations, and records the evaluation trace.
//!
//! All Fortran state is global (COMMON blocks + the shim's callback
//! registry), so every call is serialized through a process-wide mutex.
//! Inputs are validated in Rust before any Fortran code runs: the
//! upstream validation paths (IFAIL=10) print to stdout, which wrappers
//! must never trigger.  See docs/FORTRAN_INTERFACES.md for the complete
//! interface spec these wrappers follow.

use std::os::raw::{c_int, c_long, c_void};
use std::ptr;
use std::sync::Mutex;

use crate::optimizers::{OptError, OptResult};

/// Built-in objective indices (order of GM_BUILTINS in gm_shim.c).
pub const BUILTIN_FURASN: u32 = 0;
pub const BUILTIN_FUSH5: u32 = 1;
pub const BUILTIN_FUSH7: u32 = 2;
pub const BUILTIN_FUSH10: u32 = 3;
pub const BUILTIN_FUHAR3: u32 = 4;
pub const BUILTIN_FUHAR6: u32 = 5;
pub const BUILTIN_FUBRAN: u32 = 6;
pub const BUILTIN_FUGOLD: u32 = 7;

/// Map a built-in objective name (as exposed to Python/R) to its index.
pub fn builtin_index(name: &str) -> Option<u32> {
    match name {
        "furasn" => Some(BUILTIN_FURASN),
        "fush5" => Some(BUILTIN_FUSH5),
        "fush7" => Some(BUILTIN_FUSH7),
        "fush10" => Some(BUILTIN_FUSH10),
        "fuhar3" => Some(BUILTIN_FUHAR3),
        "fuhar6" => Some(BUILTIN_FUHAR6),
        "fubran" => Some(BUILTIN_FUBRAN),
        "fugold" => Some(BUILTIN_FUGOLD),
        _ => None,
    }
}

/// Objective passed to a Fortran optimizer: either one of the compiled
/// built-in test functions (never enters Rust during the run) or a Rust
/// closure invoked through the C shim callback.
pub enum FortranObjective<'a> {
    Builtin(u32),
    Closure(&'a mut dyn FnMut(&[f64]) -> f64),
}

/// Constraint callback for FLEXI/REQP: fills `r` (length n_eq + n_ineq)
/// with constraint values at `x`.  R(1..n_eq) are equalities (=0
/// feasible), the rest inequalities (>=0 feasible).
pub type FortranConstraint<'a> = &'a mut dyn FnMut(&[f64], &mut [f64]);

type GmObjectiveFn = unsafe extern "C" fn(x: *const f64, n: c_int, data: *mut c_void) -> f64;
type GmConstraintFn =
    unsafe extern "C" fn(x: *const f64, nx: c_int, r: *mut f64, k8: c_int, data: *mut c_void);

/// COMMON /LAIK/ FM, XM(100): best point found by LBAYES.
#[repr(C)]
pub struct LaikCommon {
    pub fm: f64,
    pub xm: [f64; 100],
}

/// COMMON /STATIS/: only the first word (IFAIL) has a stable meaning
/// across routines; the remaining words differ per routine (and are
/// sometimes REAL), so only `ifail` may be read.
#[repr(C)]
pub struct StatisCommon {
    pub ifail: c_int,
}

extern "C" {
    // -- C shim (fortran/gm_shim.c) --
    fn gm_set_objective(f: Option<GmObjectiveFn>, data: *mut c_void);
    fn gm_set_builtin(idx: c_int) -> c_int;
    #[allow(dead_code)]
    fn gm_n_builtins() -> c_int;
    fn gm_set_constraints(f: Option<GmConstraintFn>, data: *mut c_void);
    fn gm_reset_evals();
    fn gm_get_evals() -> c_long;
    fn gm_trace_start(cap: c_long, dim: c_int) -> c_int;
    fn gm_trace_length() -> c_long;
    fn gm_trace_dimension() -> c_int;
    fn gm_trace_points() -> *const f64;
    fn gm_trace_values() -> *const f64;
    fn gm_trace_stop();
    fn gm_ats_set(state15: *const f64);
    #[allow(dead_code)]
    fn gm_ats_get(state15: *mut f64);
    fn gm_ats_reset();

    // -- Fortran optimizer entry points (gfortran name mangling) --
    fn mig1_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn mig2_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        xn: *mut f64,
        nm: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn bayes1_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        xn: *mut f64,
        nm: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn lbayes_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        f1: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );
    fn unt_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        xn: *mut f64,
        nm: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn extr_(
        xm: *mut f64,
        bp: *const f64,
        ep: *const f64,
        ym: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );
    fn exkor_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );
    fn mivar4_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        nn: *const c_int,
        b1: *mut f64,
        nm: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );
    fn flexi_(
        z: *mut f64,
        m1: *const c_int,
        ff: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );
    fn glopt_(
        xm: *mut f64,
        a: *const f64,
        b: *const f64,
        m: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn lpmin_(
        x: *mut f64,
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        x2: *mut f64,
        nm: *const c_int,
        fmin: *mut f64,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    #[allow(dead_code)]
    fn anal1_(
        xp: *const f64,
        xg: *const f64,
        n: *const c_int,
        xx: *mut f64,
        x: *mut f64,
        nm: *const c_int,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    #[allow(dead_code)]
    fn anal2_(
        a: *const f64,
        b: *const f64,
        n: *const c_int,
        xx: *mut f64,
        x: *mut f64,
        nm: *const c_int,
        ipar: *mut c_int,
        ipa: *const c_int,
    );
    fn reqp_(
        x: *mut f64,
        b1: *mut f64,
        q: *mut f64,
        a: *mut f64,
        n: *const c_int,
        fm: *mut f64,
        ipar: *mut c_int,
        par: *const f64,
        ipa: *const c_int,
        ipaa: *const c_int,
    );

    // -- Fortran utilities --
    fn lptau_(c: *const f64, n: *const c_int, x: *mut f64);

    // -- COMMON blocks --
    static mut laik_: LaikCommon;
    static mut bs1_: [f64; 1000];
    static mut statis_: StatisCommon;
    /// ANAL2 outputs: /ANA2/ VK(20,20) eigenvectors, /ANA3/ DX(20)
    /// per-axis influence, /ANA4/ DY(20) eigen-direction influence.
    static mut ana2_: [f64; 400];
    static mut ana3_: [f64; 20];
    static mut ana4_: [f64; 20];
}

/// Result of the ANAL2 variable-influence analysis.
#[derive(Debug, Clone)]
pub struct Anal2Result {
    /// Influence of each variable in the original coordinate system.
    pub influence: Vec<f64>,
    /// Influence of each direction in the eigen coordinate system.
    pub influence_eigen: Vec<f64>,
    /// Eigenvector matrix, column-major: `eigenvectors[j]` is direction j.
    pub eigenvectors: Vec<Vec<f64>>,
}

/// Variable-influence analysis via covariance eigenstructure (original
/// Fortran ANAL2).  Does not evaluate an objective: analyses a supplied
/// design of `points` (each of dimension n) with matching `values`.
pub fn fortran_anal2(
    a: &[f64],
    b: &[f64],
    points: &[Vec<f64>],
    values: &[f64],
) -> Result<Anal2Result, OptError> {
    let n = a.len();
    let m = values.len();
    if n == 0 || n > 20 {
        return Err(OptError::InvalidInput(
            "ANAL2 requires dimension in 1..=20".to_string(),
        ));
    }
    if b.len() != n {
        return Err(OptError::InvalidInput(
            "bounds must have the same length".to_string(),
        ));
    }
    if points.len() != m || !(10..=300).contains(&m) {
        return Err(OptError::InvalidInput(
            "ANAL2 requires 10..=300 points with matching values".to_string(),
        ));
    }
    if points.iter().any(|p| p.len() != n) {
        return Err(OptError::InvalidInput(
            "every point must have the same dimension as the bounds".to_string(),
        ));
    }

    let nm = (n * m) as c_int;
    let mut xx = vec![0.0f64; n * m];
    for (i, p) in points.iter().enumerate() {
        xx[i * n..(i + 1) * n].copy_from_slice(p);
    }
    let mut xwork = vec![0.0f64; n * m];
    let mut ipar = [0 as c_int; 30];
    ipar[0] = -1;
    ipar[1] = m as c_int;
    let ipa: c_int = 0;
    let nn = n as c_int;

    let _guard = FORTRAN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        for (i, &v) in values.iter().enumerate() {
            bs1_[i] = v;
        }
        anal2_(
            a.as_ptr(),
            b.as_ptr(),
            &nn,
            xx.as_mut_ptr(),
            xwork.as_mut_ptr(),
            &nm,
            ipar.as_mut_ptr(),
            &ipa,
        );
        let ifail = ptr::addr_of!(statis_.ifail).read();
        if ifail == 10 {
            return Err(OptError::Internal(
                "ANAL2 rejected its inputs (IFAIL=10)".to_string(),
            ));
        }
        let dx = ptr::addr_of!(ana3_).read();
        let dy = ptr::addr_of!(ana4_).read();
        let vk = ptr::addr_of!(ana2_).read();
        let mut eig = Vec::with_capacity(n);
        for j in 0..n {
            eig.push((0..n).map(|i| vk[i + j * 20]).collect());
        }
        Ok(Anal2Result {
            influence: dx[..n].to_vec(),
            influence_eigen: dy[..n].to_vec(),
            eigenvectors: eig,
        })
    }
}

/// Serializes every entry into the Fortran library (COMMON blocks and
/// the shim callback registry are process-global).
static FORTRAN_LOCK: Mutex<()> = Mutex::new(());

unsafe extern "C" fn closure_trampoline(x: *const f64, n: c_int, data: *mut c_void) -> f64 {
    let cb = &mut *(data as *mut &mut dyn FnMut(&[f64]) -> f64);
    let xs = std::slice::from_raw_parts(x, n.max(0) as usize);
    cb(xs)
}

unsafe extern "C" fn constraint_trampoline(
    x: *const f64,
    nx: c_int,
    r: *mut f64,
    k8: c_int,
    data: *mut c_void,
) {
    let cb = &mut *(data as *mut &mut dyn FnMut(&[f64], &mut [f64]));
    let xs = std::slice::from_raw_parts(x, nx.max(0) as usize);
    let rs = std::slice::from_raw_parts_mut(r, k8.max(0) as usize);
    cb(xs, rs);
}

/// Trace of the run: points, values and total evaluation count.
struct RunTrace {
    points: Vec<Vec<f64>>,
    values: Vec<f64>,
    evals: usize,
}

/// Locks the library, seeds the ATS state, registers the objective (and
/// optional constraints), reserves a trace, invokes `call`, then reads
/// the trace, releases shim state and checks IFAIL.
fn run_fortran(
    objective: FortranObjective<'_>,
    constraints: Option<FortranConstraint<'_>>,
    ats_state: Option<&[f64; 15]>,
    trace_cap: usize,
    dim: usize,
    call: impl FnOnce(),
) -> Result<RunTrace, OptError> {
    let _guard = FORTRAN_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Keep the &mut dyn fat pointers alive for the duration of the call.
    let mut objective_holder: Option<&mut dyn FnMut(&[f64]) -> f64> = None;
    let mut constraint_holder: Option<&mut dyn FnMut(&[f64], &mut [f64])> = None;

    unsafe {
        match ats_state {
            Some(state) => gm_ats_set(state.as_ptr()),
            None => gm_ats_reset(),
        }
        gm_reset_evals();
        if gm_trace_start(trace_cap as c_long, dim as c_int) != 0 {
            return Err(OptError::Internal(
                "failed to allocate evaluation trace".to_string(),
            ));
        }

        match objective {
            FortranObjective::Builtin(idx) => {
                if gm_set_builtin(idx as c_int) != 0 {
                    gm_trace_stop();
                    return Err(OptError::InvalidInput(format!(
                        "unknown builtin objective index {idx}"
                    )));
                }
            }
            FortranObjective::Closure(f) => {
                objective_holder = Some(f);
                let holder = objective_holder.as_mut().unwrap();
                gm_set_objective(
                    Some(closure_trampoline),
                    holder as *mut &mut dyn FnMut(&[f64]) -> f64 as *mut c_void,
                );
            }
        }
        match constraints {
            Some(c) => {
                constraint_holder = Some(c);
                let holder = constraint_holder.as_mut().unwrap();
                gm_set_constraints(
                    Some(constraint_trampoline),
                    holder as *mut &mut dyn FnMut(&[f64], &mut [f64]) as *mut c_void,
                );
            }
            None => gm_set_constraints(None, ptr::null_mut()),
        }

        call();

        let evals = gm_get_evals().max(0) as usize;
        let len = gm_trace_length().max(0) as usize;
        let trace_dim = gm_trace_dimension().max(0) as usize;
        let pts = gm_trace_points();
        let vals = gm_trace_values();
        let mut points = Vec::with_capacity(len);
        let mut values = Vec::with_capacity(len);
        if !pts.is_null() && !vals.is_null() && trace_dim > 0 {
            for i in 0..len {
                let row = std::slice::from_raw_parts(pts.add(i * trace_dim), trace_dim);
                points.push(row.to_vec());
                values.push(*vals.add(i));
            }
        }
        gm_trace_stop();
        // Never leave dangling callback pointers registered.
        gm_set_objective(None, ptr::null_mut());
        gm_set_constraints(None, ptr::null_mut());
        drop(objective_holder);
        drop(constraint_holder);

        let ifail = ptr::addr_of!(statis_.ifail).read();
        if ifail == 10 {
            return Err(OptError::Internal(
                "Fortran routine rejected its inputs (IFAIL=10); wrapper validation is out of sync"
                    .to_string(),
            ));
        }

        Ok(RunTrace {
            points,
            values,
            evals,
        })
    }
}

/// 1-based index of the first minimum in the trace values (0 if empty).
fn best_iter_from(values: &[f64]) -> usize {
    let mut best = 0usize;
    let mut best_v = f64::INFINITY;
    for (i, &v) in values.iter().enumerate() {
        if v < best_v {
            best_v = v;
            best = i + 1;
        }
    }
    best
}

fn make_result(best_x: Vec<f64>, best_f: f64, trace: RunTrace) -> OptResult {
    OptResult {
        best_x,
        best_f,
        evals: trace.evals,
        best_iter: best_iter_from(&trace.values),
        points: trace.points,
        values: trace.values,
    }
}

fn validate_bounds(a: &[f64], b: &[f64], max_n: usize) -> Result<usize, OptError> {
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
    if a.len() > max_n {
        return Err(OptError::InvalidInput(format!(
            "dimension must be <= {max_n} for this Fortran routine"
        )));
    }
    for i in 0..a.len() {
        if a[i] > b[i] {
            return Err(OptError::InvalidInput(format!(
                "invalid interval at index {i}: a > b"
            )));
        }
    }
    Ok(a.len())
}

fn validate_start(x0: &[f64], n: usize) -> Result<(), OptError> {
    if x0.len() != n {
        return Err(OptError::InvalidInput(
            "x0 length must match bounds".to_string(),
        ));
    }
    Ok(())
}

fn ipar_new(values: &[i32]) -> [c_int; 30] {
    let mut ipar = [0 as c_int; 30];
    ipar[0] = -1; // IPR: always silent
    for (i, &v) in values.iter().enumerate() {
        ipar[i + 1] = v;
    }
    ipar
}

const IPA0: c_int = 0;

/// MIG1: pure Monte-Carlo search (ATS uniform sampling).  N<=100.
pub fn fortran_mig1(
    a: &[f64],
    b: &[f64],
    evaluations: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 100)?;
    if evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[evaluations as i32]);
    let mut x = vec![0.0f64; n];
    let mut fm = 0.0f64;
    let n_i = n as c_int;

    let trace = run_fortran(objective, None, ats_state, evaluations, n, || unsafe {
        mig1_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            &mut fm,
            ipar.as_mut_ptr(),
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// MIG2: Monte-Carlo keeping the second minimum.  N<=20, M<=1000.
pub fn fortran_mig2(
    a: &[f64],
    b: &[f64],
    evaluations: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    if evaluations == 0 || evaluations > 1000 {
        return Err(OptError::InvalidInput(
            "evaluations must be in 1..=1000".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[evaluations as i32]);
    let mut x = vec![0.0f64; n];
    let mut xn = vec![0.0f64; n * evaluations];
    let mut fm = 0.0f64;
    let n_i = n as c_int;
    let nm = (n * evaluations) as c_int;

    let trace = run_fortran(objective, None, ats_state, evaluations, n, || unsafe {
        mig2_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            xn.as_mut_ptr(),
            &nm,
            &mut fm,
            ipar.as_mut_ptr(),
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// BAYES1: one-step Bayesian search (LPTAU init + surrogate planner).
/// N<=20, M<=1000, 1<=LT<=M.
pub fn fortran_bayes1(
    a: &[f64],
    b: &[f64],
    evaluations: usize,
    initial_points: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    if evaluations == 0 || evaluations > 1000 {
        return Err(OptError::InvalidInput(
            "evaluations must be in 1..=1000".to_string(),
        ));
    }
    if initial_points == 0 || initial_points > evaluations {
        return Err(OptError::InvalidInput(
            "initial_points must be in 1..=evaluations".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[evaluations as i32, initial_points as i32]);
    let mut x = vec![0.0f64; n];
    let mut xn = vec![0.0f64; n * evaluations];
    let mut fm = 0.0f64;
    let n_i = n as c_int;
    let nm = (n * evaluations) as c_int;

    let trace = run_fortran(objective, None, ats_state, evaluations, n, || unsafe {
        bayes1_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            xn.as_mut_ptr(),
            &nm,
            &mut fm,
            ipar.as_mut_ptr(),
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// UNT: uniform deterministic grid with Wiener-model uncertainty.
/// N<=20, M<=500, ML in 1..=20; LT=0 selects the automatic value
/// max(15*N, 6*ML), otherwise 30<=LT<=M.
pub fn fortran_unt(
    a: &[f64],
    b: &[f64],
    evaluations: usize,
    initial_points: usize,
    max_local_minima: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    if evaluations == 0 || evaluations > 500 {
        return Err(OptError::InvalidInput(
            "evaluations must be in 1..=500".to_string(),
        ));
    }
    if max_local_minima == 0 || max_local_minima > 20 {
        return Err(OptError::InvalidInput(
            "max_local_minima must be in 1..=20".to_string(),
        ));
    }
    let effective_lt = if initial_points == 0 {
        (15 * n).max(6 * max_local_minima)
    } else {
        initial_points
    };
    if effective_lt < 30 || effective_lt > evaluations {
        return Err(OptError::InvalidInput(format!(
            "initial_points (effective {effective_lt}) must be in 30..=evaluations"
        )));
    }

    let mut ipar = ipar_new(&[
        evaluations as i32,
        initial_points as i32, // written back by UNT when 0
        max_local_minima as i32,
    ]);
    let mut x = vec![0.0f64; n];
    let mut xn = vec![0.0f64; n * evaluations];
    let mut fm = 0.0f64;
    let n_i = n as c_int;
    let nm = (n * evaluations) as c_int;

    let trace = run_fortran(objective, None, ats_state, evaluations + 8, n, || unsafe {
        unt_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            xn.as_mut_ptr(),
            &nm,
            &mut fm,
            ipar.as_mut_ptr(),
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// GLOPT: clustering global method.  dim<=20, 1<=PN<=150.
pub fn fortran_glopt(
    a: &[f64],
    b: &[f64],
    evaluations: usize,
    start_points: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    if evaluations == 0 {
        return Err(OptError::InvalidInput(
            "evaluations must be > 0".to_string(),
        ));
    }
    if start_points == 0 || start_points > 150 {
        return Err(OptError::InvalidInput(
            "start_points must be in 1..=150".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[evaluations as i32, start_points as i32]);
    let mut xm = vec![0.0f64; n];
    let mut fm = 0.0f64;
    let n_i = n as c_int;

    let trace = run_fortran(
        objective,
        None,
        ats_state,
        evaluations + 64,
        n,
        || unsafe {
            glopt_(
                xm.as_mut_ptr(),
                a.as_ptr(),
                b.as_ptr(),
                &n_i,
                &mut fm,
                ipar.as_mut_ptr(),
                &IPA0,
            );
        },
    )?;
    Ok(make_result(xm, fm, trace))
}

/// LPMIN: deterministic LP-tau search with optional factor analysis.
/// `analysis_evals`: <0 no analysis, 0 use the natural variable order,
/// otherwise 10..=300 analysis evaluations.  N<=20; the /BS1/ value
/// channel caps total evaluations at 1000.
pub fn fortran_lpmin(
    a: &[f64],
    b: &[f64],
    analysis_evals: i32,
    search_evals: usize,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    if search_evals == 0 {
        return Err(OptError::InvalidInput(
            "search_evals must be > 0".to_string(),
        ));
    }
    if analysis_evals > 0 && !(10..=300).contains(&analysis_evals) {
        return Err(OptError::InvalidInput(
            "analysis_evals must be <0 (none), 0 (natural order) or in 10..=300".to_string(),
        ));
    }
    let m1 = analysis_evals.max(0) as usize;
    if m1 + search_evals > 1000 {
        return Err(OptError::InvalidInput(
            "analysis_evals + search_evals must be <= 1000 (/BS1/ capacity)".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[analysis_evals, search_evals as i32]);
    if analysis_evals == 0 {
        // IPAR(IPA+4..IPA+3+N) must hold a permutation of 1..N.
        for i in 0..n {
            ipar[3 + i] = (i + 1) as c_int;
        }
    }
    let mut x = vec![0.0f64; n];
    let mut x2 = vec![0.0f64; n * m1.max(1)];
    let mut fmin = 0.0f64;
    let n_i = n as c_int;
    let nm = (n * m1.max(1)) as c_int;

    let trace = run_fortran(
        objective,
        None,
        ats_state,
        m1 + search_evals + 8,
        n,
        || unsafe {
            lpmin_(
                x.as_mut_ptr(),
                a.as_ptr(),
                b.as_ptr(),
                &n_i,
                x2.as_mut_ptr(),
                &nm,
                &mut fmin,
                ipar.as_mut_ptr(),
                &IPA0,
            );
        },
    )?;
    Ok(make_result(x, fmin, trace))
}

/// EXTR: 1-D global search on a Wiener-process model over [bp, ep].
/// M<=500, 6<=LT<=M.
#[allow(clippy::too_many_arguments)]
pub fn fortran_extr(
    bp: f64,
    ep: f64,
    evaluations: usize,
    model_evals: usize,
    acc_y: f64,
    acc_x: f64,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    if !(bp < ep) {
        return Err(OptError::InvalidInput(
            "interval must satisfy bp < ep".to_string(),
        ));
    }
    if evaluations == 0 || evaluations > 500 {
        return Err(OptError::InvalidInput(
            "evaluations must be in 1..=500".to_string(),
        ));
    }
    if model_evals < 6 || model_evals > evaluations {
        return Err(OptError::InvalidInput(
            "model_evals must be in 6..=evaluations".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[evaluations as i32, model_evals as i32]);
    let mut par = [0.0f64; 30];
    par[0] = acc_y;
    par[1] = acc_x;
    let mut xm = 0.0f64;
    let mut ym = 0.0f64;

    let trace = run_fortran(objective, None, ats_state, evaluations + 8, 1, || unsafe {
        extr_(
            &mut xm,
            &bp,
            &ep,
            &mut ym,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;
    Ok(make_result(vec![xm], ym, trace))
}

/// EXKOR: n-D coordinate optimization; every coordinate is solved by an
/// EXTR-style 1-D Wiener search.  N<=20, M<=500 per 1-D search,
/// 6<=LT<=M, 1<=first_coord<=N.
#[allow(clippy::too_many_arguments)]
pub fn fortran_exkor(
    x0: &[f64],
    a: &[f64],
    b: &[f64],
    evals_per_coord: usize,
    model_evals: usize,
    cycles: usize,
    first_coord: usize,
    acc: f64,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 20)?;
    validate_start(x0, n)?;
    if evals_per_coord == 0 || evals_per_coord > 500 {
        return Err(OptError::InvalidInput(
            "evals_per_coord must be in 1..=500".to_string(),
        ));
    }
    if model_evals < 6 || model_evals > evals_per_coord {
        return Err(OptError::InvalidInput(
            "model_evals must be in 6..=evals_per_coord".to_string(),
        ));
    }
    if cycles == 0 {
        return Err(OptError::InvalidInput("cycles must be > 0".to_string()));
    }
    if first_coord == 0 || first_coord > n {
        return Err(OptError::InvalidInput(
            "first_coord must be in 1..=n".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[
        evals_per_coord as i32,
        model_evals as i32,
        cycles as i32,
        first_coord as i32,
    ]);
    let mut par = [0.0f64; 30];
    par[0] = acc; // accuracy of FM
    for i in 0..n {
        par[1 + i] = acc; // per-coordinate accuracy
    }
    let mut x = x0.to_vec();
    let mut fm = 0.0f64;
    let n_i = n as c_int;

    let cap = evals_per_coord * n * cycles + 16;
    let trace = run_fortran(objective, None, ats_state, cap, n, || unsafe {
        exkor_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            &mut fm,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// MIVAR4: variable-metric local method with numeric gradients.  N<=100.
#[allow(clippy::too_many_arguments)]
pub fn fortran_mivar4(
    x0: &[f64],
    a: &[f64],
    b: &[f64],
    max_evals: usize,
    nstop: usize,
    imax: usize,
    xeps: f64,
    eps: f64,
    eps1: f64,
    delta: f64,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 100)?;
    validate_start(x0, n)?;
    if max_evals == 0 {
        return Err(OptError::InvalidInput("max_evals must be > 0".to_string()));
    }

    let mut ipar = ipar_new(&[max_evals as i32, nstop as i32, imax as i32]);
    let par = {
        let mut p = [0.0f64; 30];
        p[0] = xeps;
        p[1] = eps;
        p[2] = eps1;
        p[3] = delta;
        p
    };
    let mut x = x0.to_vec();
    let nm = n * (n + 1) / 2;
    let mut b1 = vec![0.0f64; nm];
    let mut fm = 0.0f64;
    let n_i = n as c_int;
    let nm_i = nm as c_int;

    let cap = max_evals + 2 * (n + 1) + 8;
    let trace = run_fortran(objective, None, ats_state, cap, n, || unsafe {
        mivar4_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            b1.as_mut_ptr(),
            &nm_i,
            &mut fm,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// LBAYES: Bayesian method with local descent (deterministic).  N<=100,
/// IT>0.  The best point is reported through COMMON /LAIK/; `best_f` is
/// the smoothed mean value the method minimizes.
#[allow(clippy::too_many_arguments)]
pub fn fortran_lbayes(
    x0: &[f64],
    a: &[f64],
    b: &[f64],
    iterations: usize,
    aniu: f64,
    beta: f64,
    objective: FortranObjective<'_>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = validate_bounds(a, b, 100)?;
    validate_start(x0, n)?;
    if iterations == 0 {
        return Err(OptError::InvalidInput("iterations must be > 0".to_string()));
    }

    let mut ipar = ipar_new(&[iterations as i32, 0]); // NIPA=0: no integer variables
    let par = {
        let mut p = [0.0f64; 30];
        p[0] = aniu;
        p[1] = beta;
        p
    };
    let mut x = x0.to_vec();
    let mut f1 = 0.0f64;
    let n_i = n as c_int;

    let cap = 2 * n + iterations * (2 * n + 2) + 8;
    let trace = run_fortran(objective, None, ats_state, cap, n, || unsafe {
        lbayes_(
            x.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            &n_i,
            &mut f1,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;

    // Best point and value live in COMMON /LAIK/ (already unscaled).
    let (best_f, best_x) = unsafe {
        let fm = ptr::addr_of!(laik_.fm).read();
        let xm = ptr::addr_of!(laik_.xm).read();
        (fm, xm[..n].to_vec())
    };
    Ok(make_result(best_x, best_f, trace))
}

/// FLEXI: flexible-tolerance Nelder-Mead simplex with equality and
/// inequality constraints.  dim<=20, n_eq+n_ineq<=100.  There are NO
/// bounds: the search region is set by the start point and `size`.
#[allow(clippy::too_many_arguments)]
pub fn fortran_flexi(
    x0: &[f64],
    max_evals: usize,
    n_eq: usize,
    n_ineq: usize,
    size: f64,
    conver: f64,
    objective: FortranObjective<'_>,
    constraints: Option<FortranConstraint<'_>>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = x0.len();
    if n == 0 || n > 20 {
        return Err(OptError::InvalidInput(
            "dimension must be in 1..=20".to_string(),
        ));
    }
    if max_evals == 0 {
        return Err(OptError::InvalidInput("max_evals must be > 0".to_string()));
    }
    if n_eq + n_ineq > 100 {
        return Err(OptError::InvalidInput(
            "n_eq + n_ineq must be <= 100".to_string(),
        ));
    }
    if n_eq >= n {
        return Err(OptError::InvalidInput(
            "n_eq must be < dimension".to_string(),
        ));
    }
    if n_eq + n_ineq > 0 && constraints.is_none() {
        return Err(OptError::InvalidInput(
            "constraints callback required when n_eq + n_ineq > 0".to_string(),
        ));
    }
    if size <= 0.0 {
        return Err(OptError::InvalidInput("size must be > 0".to_string()));
    }

    let mut ipar = ipar_new(&[max_evals as i32, n_eq as i32, n_ineq as i32]);
    let par = {
        let mut p = [0.0f64; 30];
        p[0] = size;
        p[1] = conver;
        p
    };
    let mut z = x0.to_vec();
    let mut ff = 0.0f64;
    let m1 = n as c_int;

    let cap = max_evals + 8 * (n + 5);
    let trace = run_fortran(objective, constraints, ats_state, cap, n, || unsafe {
        flexi_(
            z.as_mut_ptr(),
            &m1,
            &mut ff,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;
    Ok(make_result(z, ff, trace))
}

/// REQP: recursive quadratic programming with constraints.  N<=100,
/// n_eq+n_ineq<=100.  No bounds; constraints share FLEXI's sign
/// convention.
#[allow(clippy::too_many_arguments)]
pub fn fortran_reqp(
    x0: &[f64],
    imax: usize,
    n_eq: usize,
    n_ineq: usize,
    r1: f64,
    scale: f64,
    delta: f64,
    eps: f64,
    objective: FortranObjective<'_>,
    constraints: Option<FortranConstraint<'_>>,
    ats_state: Option<&[f64; 15]>,
) -> Result<OptResult, OptError> {
    let n = x0.len();
    if n == 0 || n > 100 {
        return Err(OptError::InvalidInput(
            "dimension must be in 1..=100".to_string(),
        ));
    }
    if imax == 0 {
        return Err(OptError::InvalidInput("imax must be > 0".to_string()));
    }
    if n_eq + n_ineq > 100 {
        return Err(OptError::InvalidInput(
            "n_eq + n_ineq must be <= 100".to_string(),
        ));
    }
    if n_eq + n_ineq > 0 && constraints.is_none() {
        return Err(OptError::InvalidInput(
            "constraints callback required when n_eq + n_ineq > 0".to_string(),
        ));
    }

    let mut ipar = ipar_new(&[imax as i32, n_eq as i32, n_ineq as i32]);
    let par = {
        let mut p = [0.0f64; 30];
        p[0] = r1;
        p[1] = scale;
        p[2] = delta;
        p[3] = eps;
        p
    };
    let mut x = x0.to_vec();
    let mut b1 = vec![0.0f64; n * n];
    let mut q = vec![0.0f64; n * n];
    let mut amat = vec![0.0f64; 100 * n];
    let mut fm = 0.0f64;
    let n_i = n as c_int;

    let cap = imax * (n + 2) * 8 + 64;
    let trace = run_fortran(objective, constraints, ats_state, cap, n, || unsafe {
        reqp_(
            x.as_mut_ptr(),
            b1.as_mut_ptr(),
            q.as_mut_ptr(),
            amat.as_mut_ptr(),
            &n_i,
            &mut fm,
            ipar.as_mut_ptr(),
            par.as_ptr(),
            &IPA0,
            &IPA0,
        );
    })?;
    Ok(make_result(x, fm, trace))
}

/// LPTAU: point `c` (1-based) of the n-dimensional LP-tau sequence.
pub fn fortran_lptau(c: usize, n: usize) -> Result<Vec<f64>, OptError> {
    if c == 0 {
        return Err(OptError::InvalidInput(
            "point number must be >= 1".to_string(),
        ));
    }
    if n == 0 || n > 20 {
        return Err(OptError::InvalidInput(
            "dimension must be in 1..=20".to_string(),
        ));
    }
    let _guard = FORTRAN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut x = vec![0.0f64; n];
    let c_f = c as f64;
    let n_i = n as c_int;
    unsafe {
        lptau_(&c_f, &n_i, x.as_mut_ptr());
    }
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks;

    const A2: [f64; 2] = [-0.25, -0.125];
    const B2: [f64; 2] = [0.5, 0.625];

    #[test]
    fn lptau_reference_points() {
        let expected = [
            [0.5, 0.5],
            [0.25, 0.75],
            [0.75, 0.25],
            [0.125, 0.625],
        ];
        for (i, exp) in expected.iter().enumerate() {
            let p = fortran_lptau(i + 1, 2).unwrap();
            for j in 0..2 {
                assert!(
                    (p[j] - exp[j]).abs() < 1e-15,
                    "LPTAU point {} coordinate {}: got {}, want {}",
                    i + 1,
                    j,
                    p[j],
                    exp[j]
                );
            }
        }
    }

    #[test]
    fn bayes1_builtin_reference() {
        let result = fortran_bayes1(
            &A2,
            &B2,
            200,
            20,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        assert!(
            (result.best_f - (-1.9966149827762463)).abs() < 1e-12,
            "best_f = {:.17}",
            result.best_f
        );
        assert_eq!(result.evals, 200);
        assert_eq!(result.values.len(), 200);
        // First recorded values from docs/FORTRAN_INTERFACES.md.
        let first = [
            0.91709442215351888,
            -0.21484719829221782,
            -1.1207175656694048,
        ];
        for (i, exp) in first.iter().enumerate() {
            assert!(
                (result.values[i] - exp).abs() < 1e-12,
                "values[{}] = {:.17}",
                i,
                result.values[i]
            );
        }
        assert!(result.best_iter >= 1 && result.best_iter <= 200);
    }

    #[test]
    fn mig2_builtin_reference() {
        let result = fortran_mig2(
            &A2,
            &B2,
            200,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        assert!(
            (result.best_f - (-1.8296951132230181)).abs() < 1e-12,
            "best_f = {:.17}",
            result.best_f
        );
        assert_eq!(result.evals, 200);
    }

    #[test]
    fn closure_objective_matches_builtin() {
        let builtin = fortran_bayes1(
            &A2,
            &B2,
            200,
            20,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();

        let mut evals_seen = 0usize;
        let mut obj = |x: &[f64]| {
            evals_seen += 1;
            benchmarks::furasn(x)
        };
        let closure = fortran_bayes1(
            &A2,
            &B2,
            200,
            20,
            FortranObjective::Closure(&mut obj),
            None,
        )
        .unwrap();

        assert_eq!(evals_seen, 200);
        // Identical best_f proves the callback path feeds the optimizer
        // the same values as the compiled builtin.  (Individual trace
        // values may differ by 1 ULP: gfortran's COS vs Rust's cos.)
        assert_eq!(builtin.best_f, closure.best_f);
        assert_eq!(builtin.best_x, closure.best_x);
        assert_eq!(builtin.values.len(), closure.values.len());
        for (bv, cv) in builtin.values.iter().zip(closure.values.iter()) {
            assert!((bv - cv).abs() < 1e-12, "trace diverged: {bv} vs {cv}");
        }
    }

    #[test]
    fn ats_seed_changes_and_reproduces_mig2() {
        let default1 = fortran_mig2(
            &A2,
            &B2,
            200,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        let seed = [
            0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77, 0.88, 0.99, 0.12, 0.23, 0.34, 0.45, 0.56,
            0.67,
        ];
        let seeded1 = fortran_mig2(
            &A2,
            &B2,
            200,
            FortranObjective::Builtin(BUILTIN_FURASN),
            Some(&seed),
        )
        .unwrap();
        let seeded2 = fortran_mig2(
            &A2,
            &B2,
            200,
            FortranObjective::Builtin(BUILTIN_FURASN),
            Some(&seed),
        )
        .unwrap();
        assert_ne!(default1.best_f, seeded1.best_f);
        assert_eq!(seeded1.best_f, seeded2.best_f);
        assert_eq!(seeded1.best_x, seeded2.best_x);
    }

    #[test]
    fn invalid_inputs_rejected_before_fortran() {
        // Dimension too large for MIG2 (N<=20).
        let a = vec![0.0; 21];
        let b = vec![1.0; 21];
        assert!(matches!(
            fortran_mig2(&a, &b, 100, FortranObjective::Builtin(BUILTIN_FURASN), None),
            Err(OptError::InvalidInput(_))
        ));
        // Budget above /BS1/-backed limit for BAYES1 (M<=1000).
        assert!(matches!(
            fortran_bayes1(
                &A2,
                &B2,
                1001,
                20,
                FortranObjective::Builtin(BUILTIN_FURASN),
                None
            ),
            Err(OptError::InvalidInput(_))
        ));
        // LT out of range for UNT.
        assert!(matches!(
            fortran_unt(
                &A2,
                &B2,
                100,
                10,
                5,
                FortranObjective::Builtin(BUILTIN_FURASN),
                None
            ),
            Err(OptError::InvalidInput(_))
        ));
    }

    #[test]
    fn extr_one_dimensional() {
        let mut obj = |x: &[f64]| benchmarks::furasn(x);
        let result = fortran_extr(
            -0.25,
            0.5,
            100,
            6,
            0.01,
            0.01,
            FortranObjective::Closure(&mut obj),
            None,
        )
        .unwrap();
        assert_eq!(result.best_x.len(), 1);
        assert!(result.evals > 0 && result.evals <= 108);
        // FURASN in 1-D has its global minimum -2 at x=0.
        assert!(result.best_f < -1.8, "best_f = {}", result.best_f);
    }

    #[test]
    fn glopt_unt_lpmin_run() {
        let g = fortran_glopt(
            &A2,
            &B2,
            300,
            10,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        assert!(g.best_f < -1.5, "glopt best_f = {}", g.best_f);

        let u = fortran_unt(
            &A2,
            &B2,
            120,
            0,
            5,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        assert!(u.best_f < -1.0, "unt best_f = {}", u.best_f);

        let l = fortran_lpmin(
            &A2,
            &B2,
            50,
            100,
            FortranObjective::Builtin(BUILTIN_FURASN),
            None,
        )
        .unwrap();
        assert!(l.best_f < -1.5, "lpmin best_f = {}", l.best_f);
    }

    #[test]
    fn local_and_constrained_methods_run() {
        let x0 = [0.3, 0.4];
        let a = [-1.0, -1.0];
        let b = [1.0, 1.0];

        let mut obj = |x: &[f64]| benchmarks::furasn(x);
        let ek = fortran_exkor(
            &x0,
            &a,
            &b,
            100,
            6,
            2,
            1,
            0.01,
            FortranObjective::Closure(&mut obj),
            None,
        )
        .unwrap();
        assert!(ek.best_f < -1.5, "exkor best_f = {}", ek.best_f);

        let mut quad = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let mv = fortran_mivar4(
            &x0,
            &a,
            &b,
            100,
            2,
            100,
            100.0,
            1e-4,
            1e-4,
            1e-4,
            FortranObjective::Closure(&mut quad),
            None,
        )
        .unwrap();
        assert!(mv.best_f < 1e-6, "mivar4 best_f = {}", mv.best_f);

        let mut quad2 = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let lb = fortran_lbayes(
            &x0,
            &a,
            &b,
            5,
            0.05,
            0.9,
            FortranObjective::Closure(&mut quad2),
            None,
        )
        .unwrap();
        assert_eq!(lb.best_x.len(), 2);
        assert!(lb.best_f.is_finite());

        // FLEXI: minimize x^2+y^2 subject to x + y >= 1 (one inequality).
        let mut quad3 = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let mut con = |x: &[f64], r: &mut [f64]| {
            r[0] = x[0] + x[1] - 1.0;
        };
        let fx = fortran_flexi(
            &x0,
            200,
            0,
            1,
            0.3,
            1e-6,
            FortranObjective::Closure(&mut quad3),
            Some(&mut con),
            None,
        )
        .unwrap();
        assert!(
            fx.best_x[0] + fx.best_x[1] > 0.99,
            "flexi solution infeasible: {:?}",
            fx.best_x
        );
        assert!((fx.best_f - 0.5).abs() < 0.05, "flexi best_f = {}", fx.best_f);

        // REQP on the same problem.
        let mut quad4 = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let mut con2 = |x: &[f64], r: &mut [f64]| {
            r[0] = x[0] + x[1] - 1.0;
        };
        let rq = fortran_reqp(
            &x0,
            50,
            0,
            1,
            1.0,
            0.25,
            1e-4,
            1e-4,
            FortranObjective::Closure(&mut quad4),
            Some(&mut con2),
            None,
        )
        .unwrap();
        assert!(
            (rq.best_f - 0.5).abs() < 1e-3,
            "reqp best_f = {}",
            rq.best_f
        );
    }
}
