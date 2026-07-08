# Wrappers for the remaining GlobalMinimum routines (Fortran backend, with
# reference-backend dispatch where a pure-R reference implementation
# exists).  See docs/FORTRAN_INTERFACES.md in the source repository for the
# underlying IPAR/PAR layouts and limits.

#' Bayesian global search with local refinement (LBAYES)
#'
#' Bayesian coordinate search by stochastic approximation with decreasing
#' step sizes (routine `LBAYES`).  Runs `iterations` sweeps from the
#' starting point `x0`; the box `[a, b]` is handled by projection.
#'
#' @inheritParams mig1
#' @param x0 Numeric starting point (defaults to the box center).
#' @param iterations Number of iterations (each performs several objective
#'   evaluations).
#' @param aniu Rate of trial-step decrease (default 0.05, the upstream
#'   example value).
#' @param beta Rate of iteration-step decrease (default 0.9).
#' @inherit mig1 return
#' @examples
#' r <- lbayes(c(-1, -1), c(1, 1), iterations = 20, objective = "furasn")
#' r$best_f
#' @export
lbayes <- function(a, b, iterations, x0 = (a + b) / 2, aniu = 0.05,
                   beta = 0.9, objective = "furasn",
                   backend = c("fortran", "reference"),
                   trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    res <- ref_lbayes(a, b, as.integer(iterations) * 10L,
                      max(2L, as.integer(iterations)), as.integer(iterations), fn)
    return(.gm_result_ref(res, "lbayes"))
  }
  .validate_bounds_fortran(a, b, 100L)
  iterations <- as.integer(iterations)
  if (iterations <= 0L) stop("iterations must be > 0")
  x0 <- .check_start(x0, a, b)
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_lbayes, x0, a, b, iterations, as.double(aniu),
               as.double(beta), o$obj, o$rho, isTRUE(trace),
               .ats_seed_state(seed))
  .gm_result(res, "lbayes", "fortran")
}

#' Multi-start Bayesian global minimization (UNT)
#'
#' Uniform random multi-start search with a Wiener-model uncertainty
#' criterion collecting up to `max_minima` local minima (routine `UNT`).
#'
#' @inheritParams mig1
#' @param evaluations Total evaluation budget (Fortran limit: 500).
#' @param initial_points Number of initial random points; `0` (default) lets
#'   the routine choose `max(15 * n, 6 * max_minima)`; otherwise must be in
#'   `30..evaluations`.
#' @param max_minima Maximum number of local minima to collect (1..20).
#' @inherit mig1 return
#' @examples
#' r <- unt(c(-1, -1), c(1, 1), 200, objective = "furasn")
#' r$best_f
#' @export
unt <- function(a, b, evaluations, initial_points = 0L, max_minima = 5L,
                objective = "furasn", backend = c("fortran", "reference"),
                trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_unt(a, b, as.integer(evaluations), 0.15, fn), "unt"))
  }
  .validate_bounds_fortran(a, b)
  evaluations <- as.integer(evaluations)
  initial_points <- as.integer(initial_points)
  max_minima <- as.integer(max_minima)
  if (evaluations <= 0L || evaluations > 500L) stop("evaluations must be in 1..500")
  if (initial_points != 0L && (initial_points < 30L || initial_points > evaluations)) {
    stop("initial_points must be 0 (auto) or in 30..evaluations")
  }
  if (max_minima < 1L || max_minima > 20L) stop("max_minima must be in 1..20")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_unt, a, b, evaluations, initial_points, max_minima,
               o$obj, o$rho, isTRUE(trace), .ats_seed_state(seed))
  .gm_result(res, "unt", "fortran")
}

#' Clustering global minimization (GLOPT)
#'
#' Random-sampling global method that clusters promising points and runs
#' local random searches within clusters (routine `GLOPT`).
#'
#' @inheritParams mig1
#' @param start_points Number of sampled starting points (1..150).
#' @inherit mig1 return
#' @examples
#' r <- glopt(c(-1, -1), c(1, 1), 500, objective = "furasn")
#' r$best_f
#' @export
glopt <- function(a, b, evaluations, start_points = 10L,
                  objective = "furasn", backend = c("fortran", "reference"),
                  trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    init <- max(2L, min(20L, as.integer(evaluations) %/% 5L))
    return(.gm_result_ref(ref_glopt(a, b, as.integer(evaluations), init, 6L, 0.92, fn), "glopt"))
  }
  .validate_bounds_fortran(a, b)
  evaluations <- as.integer(evaluations)
  start_points <- as.integer(start_points)
  if (evaluations <= 0L) stop("evaluations must be > 0")
  if (start_points < 1L || start_points > 150L) stop("start_points must be in 1..150")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_glopt, a, b, evaluations, start_points, o$obj, o$rho,
               isTRUE(trace), .ats_seed_state(seed))
  .gm_result(res, "glopt", "fortran")
}

#' LP-tau deterministic global search (LPMIN)
#'
#' Deterministic search over LP-tau (Sobol-type) points, optionally preceded
#' by a sensitivity-analysis phase that reorders variables by influence
#' (routine `LPMIN`).
#'
#' @inheritParams mig1
#' @param analysis_evals Evaluations for the analysis phase: negative for
#'   none, or a value in `10..300`.
#' @param search_evals Evaluations for the search phase.
#' @inherit mig1 return
#' @examples
#' r <- lpmin(c(-1, -1), c(1, 1), 50, 100, objective = "furasn")
#' r$best_f
#' @export
lpmin <- function(a, b, analysis_evals, search_evals, objective = "furasn",
                  backend = c("fortran", "reference"),
                  trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_lpmin(a, b, max(0L, as.integer(analysis_evals)),
                                    as.integer(search_evals), fn), "lpmin"))
  }
  .validate_bounds_fortran(a, b)
  analysis_evals <- as.integer(analysis_evals)
  search_evals <- as.integer(search_evals)
  if (analysis_evals >= 0L && (analysis_evals < 10L || analysis_evals > 300L)) {
    if (analysis_evals != 0L) {
      stop("analysis_evals must be negative (none) or in 10..300")
    }
    # analysis_evals == 0 would require an a-priori variable order; use none
    analysis_evals <- -1L
  }
  if (search_evals <= 0L) stop("search_evals must be > 0")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_lpmin, a, b, analysis_evals, search_evals, o$obj, o$rho,
               isTRUE(trace), .ats_seed_state(seed))
  .gm_result(res, "lpmin", "fortran")
}

#' One-dimensional Bayesian global minimization (EXTR)
#'
#' Global minimization of a one-dimensional function on an interval using a
#' Wiener-process model (routine `EXTR`).
#'
#' @inheritParams mig1
#' @param bp,ep Interval endpoints.
#' @param evaluations Maximum evaluations (Fortran limit: 500).
#' @param model_evals Evaluations used to estimate the Wiener model
#'   parameters (>= 6).
#' @param acc_y,acc_x Requested accuracy of the optimum value and location.
#' @inherit mig1 return
#' @examples
#' r <- extr(-1, 1, 50, objective = function(x) (x[1] - 0.3)^2)
#' r$best_x
#' @export
extr <- function(bp, ep, evaluations, model_evals = 6L, acc_y = 1e-4,
                 acc_x = 1e-4, objective = "furasn", trace = TRUE) {
  evaluations <- as.integer(evaluations)
  model_evals <- as.integer(model_evals)
  if (evaluations <= 0L || evaluations > 500L) stop("evaluations must be in 1..500")
  if (model_evals < 6L || model_evals > evaluations) {
    stop("model_evals must be in 6..evaluations")
  }
  if (!is.finite(bp) || !is.finite(ep) || bp == ep) stop("invalid interval")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_extr, as.double(bp), as.double(ep), evaluations,
               model_evals, as.double(acc_y), as.double(acc_x), o$obj,
               o$rho, isTRUE(trace))
  .gm_result(res, "extr", "fortran")
}

#' Coordinate-wise Bayesian optimization (EXKOR)
#'
#' Coordinate descent where each one-dimensional subproblem is solved by the
#' Wiener-model global method of [extr()] (routine `EXKOR`).
#'
#' @inheritParams mig1
#' @param x0 Starting point.
#' @param evaluations Maximum evaluations per one-dimensional search
#'   (Fortran limit: 500).
#' @param model_evals Evaluations for model estimation in each
#'   one-dimensional search (>= 6).
#' @param cycles Number of full coordinate cycles.
#' @param first_coord Index of the coordinate optimized first.
#' @param acc Requested accuracy (applied to the value and each coordinate).
#' @inherit mig1 return
#' @examples
#' r <- exkor(c(-1, -1), c(1, 1), 50, x0 = c(0.1, 0.1), objective = "furasn")
#' r$best_f
#' @export
exkor <- function(a, b, evaluations, x0 = (a + b) / 2, model_evals = 6L,
                  cycles = 2L, first_coord = 1L, acc = 1e-2,
                  objective = "furasn", backend = c("fortran", "reference"),
                  trace = TRUE) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_exkor(.check_start(x0, a, b), a, b,
                                    as.integer(evaluations), 0.25, 0.8, fn), "exkor"))
  }
  .validate_bounds_fortran(a, b)
  n <- length(a)
  if (n + 1L > 30L) stop("dimension too large for EXKOR PAR array")
  evaluations <- as.integer(evaluations)
  model_evals <- as.integer(model_evals)
  cycles <- as.integer(cycles)
  first_coord <- as.integer(first_coord)
  if (evaluations <= 0L || evaluations > 500L) stop("evaluations must be in 1..500")
  if (model_evals < 6L || model_evals > evaluations) stop("model_evals must be in 6..evaluations")
  if (cycles < 1L) stop("cycles must be >= 1")
  if (first_coord < 1L || first_coord > n) stop("first_coord must be in 1..length(a)")
  x0 <- .check_start(x0, a, b)
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_exkor, x0, a, b, evaluations, model_evals, cycles,
               first_coord, as.double(acc), o$obj, o$rho, isTRUE(trace))
  .gm_result(res, "exkor", "fortran")
}

#' Variable-metric local minimization on a box (MIVAR4)
#'
#' Quasi-Newton local method with finite-difference gradients and active
#' bound handling (routine `MIVAR4`).
#'
#' @inheritParams mig1
#' @param x0 Starting point.
#' @param evaluations Maximum objective evaluations.
#' @param nstop Number of consecutive small-change iterations before
#'   stopping.
#' @param imax Maximum iterations.
#' @param xeps,eps,eps1,delta Step, gradient, function-change tolerances and
#'   finite-difference step (upstream example defaults).
#' @inherit mig1 return
#' @examples
#' r <- mivar4(c(-1, -1), c(1, 1), 100, x0 = c(0.2, 0.2), objective = "furasn")
#' r$best_f
#' @export
mivar4 <- function(a, b, evaluations, x0 = (a + b) / 2, nstop = 2L,
                   imax = 100L, xeps = 100, eps = 1e-4, eps1 = 1e-4,
                   delta = 1e-4, objective = "furasn",
                   backend = c("fortran", "reference"), trace = TRUE) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_mivar4(.check_start(x0, a, b), a, b,
                                     as.integer(evaluations), 0.1, fn), "mivar4"))
  }
  .validate_bounds_fortran(a, b, 100L)
  evaluations <- as.integer(evaluations)
  if (evaluations <= 0L) stop("evaluations must be > 0")
  if (as.integer(nstop) <= 0L || as.integer(imax) <= 0L) stop("nstop and imax must be > 0")
  x0 <- .check_start(x0, a, b)
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_mivar4, x0, a, b, evaluations, as.integer(nstop),
               as.integer(imax), as.double(xeps), as.double(eps),
               as.double(eps1), as.double(delta), o$obj, o$rho,
               isTRUE(trace))
  .gm_result(res, "mivar4", "fortran")
}

#' Flexible-tolerance constrained simplex (FLEXI)
#'
#' Nelder-Mead-type simplex for problems with equality and inequality
#' constraints (routine `FLEXI`).  There are no bound arguments: the search
#' region is determined by the starting point and the initial simplex edge
#' length `size`; add bounds as inequality constraints if needed.
#'
#' @inheritParams mig1
#' @param x0 Starting point.
#' @param evaluations Maximum objective evaluations.
#' @param constraints `NULL` for unconstrained, or a function `g(x)`
#'   returning a numeric vector: first `n_eq` components are equalities
#'   (feasible when 0), the remaining `n_ineq` are inequalities (feasible
#'   when >= 0).
#' @param n_eq,n_ineq Number of equality/inequality constraint components
#'   returned by `constraints`.
#' @param size Edge length of the initial simplex.
#' @param conver Convergence tolerance.
#' @inherit mig1 return
#' @examples
#' r <- flexi(c(0.2, 0.2), 200, objective = "furasn")
#' r$best_f
#' @export
flexi <- function(x0, evaluations, constraints = NULL, n_eq = 0L,
                  n_ineq = 0L, size = 0.3, conver = 1e-5,
                  objective = "furasn", trace = TRUE) {
  x0 <- as.double(x0)
  n <- length(x0)
  if (n < 1L || n > 20L) stop("dimension must be in 1..20")
  evaluations <- as.integer(evaluations)
  n_eq <- as.integer(n_eq); n_ineq <- as.integer(n_ineq)
  if (evaluations <= 0L) stop("evaluations must be > 0")
  if (n_eq < 0L || n_ineq < 0L || n_eq + n_ineq > 100L) {
    stop("constraint counts must be >= 0 with n_eq + n_ineq <= 100")
  }
  if ((n_eq + n_ineq > 0L) && !is.function(constraints)) {
    stop("constraints function required when n_eq + n_ineq > 0")
  }
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_flexi, x0, evaluations, n_eq, n_ineq, as.double(size),
               as.double(conver), o$obj, o$rho,
               if (is.function(constraints)) constraints else NULL,
               isTRUE(trace))
  .gm_result(res, "flexi", "fortran")
}

#' Recursive quadratic programming (REQP)
#'
#' Constrained nonlinear minimization by recursive quadratic programming
#' with BFGS updates and finite-difference gradients (routine `REQP`).
#'
#' @inheritParams flexi
#' @param imax Maximum iterations.
#' @param r1 Penalty parameter.
#' @param scale Scaling parameter.
#' @param delta Finite-difference gradient step.
#' @param eps Tolerance level.
#' @inherit mig1 return
#' @examples
#' r <- reqp(c(0.2, 0.2), 50, objective = "furasn")
#' r$best_f
#' @export
reqp <- function(x0, imax, constraints = NULL, n_eq = 0L, n_ineq = 0L,
                 r1 = 1, scale = 0.25, delta = 1e-4, eps = 1e-4,
                 objective = "furasn", trace = TRUE) {
  x0 <- as.double(x0)
  n <- length(x0)
  if (n < 1L || n > 100L) stop("dimension must be in 1..100")
  imax <- as.integer(imax)
  n_eq <- as.integer(n_eq); n_ineq <- as.integer(n_ineq)
  if (imax < 0L) stop("imax must be >= 0")
  if (n_eq < 0L || n_ineq < 0L || n_eq + n_ineq > 100L) {
    stop("constraint counts must be >= 0 with n_eq + n_ineq <= 100")
  }
  if ((n_eq + n_ineq > 0L) && !is.function(constraints)) {
    stop("constraints function required when n_eq + n_ineq > 0")
  }
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_reqp, x0, imax, n_eq, n_ineq, as.double(r1),
               as.double(scale), as.double(delta), as.double(eps), o$obj,
               o$rho, if (is.function(constraints)) constraints else NULL,
               isTRUE(trace))
  .gm_result(res, "reqp", "fortran")
}

#' Selection of dominant variables and interactions (ANAL1)
#'
#' Harmonic-analysis screening of variable influence from a sample of
#' points and objective values (routine `ANAL1`).  The routine does not
#' evaluate the objective itself: supply a design (for example the trace of
#' a [mig2()] run) and its values.
#'
#' @param a,b Numeric vectors of lower and upper bounds.
#' @param points Matrix (`m` rows, `n` columns) of sample points in
#'   `[a, b]`; `m` must be in `10..300`.
#' @param values Numeric vector of the `m` objective values.
#' @param harmonics Number of harmonics investigated (1..7).
#' @param max_selected Maximum number of selected variables/interactions
#'   (1..30).
#' @param interactions If `TRUE`, consider variable pairs as well as single
#'   variables.
#' @return A list with `influence` (sorted influence measures),
#'   `variables` (matrix of 1-based variable indices; second column 0 for
#'   single variables) and `status` (Fortran IFAIL code, 0 = OK).
#' @examples
#' r <- mig2(c(-1, -1), c(1, 1), 100, "furasn")
#' anal1(c(-1, -1), c(1, 1), r$points, r$values)
#' @export
anal1 <- function(a, b, points, values, harmonics = 7L, max_selected = 8L,
                  interactions = TRUE) {
  a <- as.double(a); b <- as.double(b)
  .validate_bounds_fortran(a, b)
  points <- as.matrix(points)
  m <- nrow(points)
  if (ncol(points) != length(a)) stop("points must have length(a) columns")
  if (length(values) != m) stop("values must match nrow(points)")
  if (m < 10L || m > 300L) stop("number of points must be in 10..300")
  harmonics <- as.integer(harmonics)
  max_selected <- as.integer(max_selected)
  if (harmonics < 1L || harmonics > 7L) stop("harmonics must be in 1..7")
  if (max_selected < 1L || max_selected > 30L) stop("max_selected must be in 1..30")
  .Call(C_gm_anal1, a, b, points, as.double(values), harmonics,
        max_selected, if (isTRUE(interactions)) 2L else 1L)
}

#' Variable-influence analysis via covariance eigenstructure (ANAL2)
#'
#' Influence analysis of variables from a sample of points and objective
#' values via a factor/covariance decomposition (routine `ANAL2`).  Like
#' [anal1()], the objective is not evaluated.
#'
#' @inheritParams anal1
#' @return A list with `influence` (per-variable influence in the original
#'   coordinates), `influence_eigen` (influence in the eigen coordinate
#'   system), `eigenvectors` (n x n matrix) and `status` (IFAIL, 0 = OK).
#' @examples
#' r <- mig2(c(-1, -1), c(1, 1), 100, "furasn")
#' anal2(c(-1, -1), c(1, 1), r$points, r$values)
#' @export
anal2 <- function(a, b, points, values) {
  a <- as.double(a); b <- as.double(b)
  .validate_bounds_fortran(a, b)
  points <- as.matrix(points)
  m <- nrow(points)
  if (ncol(points) != length(a)) stop("points must have length(a) columns")
  if (length(values) != m) stop("values must match nrow(points)")
  if (m < 10L || m > 300L) stop("number of points must be in 10..300")
  .Call(C_gm_anal2, a, b, points, as.double(values))
}

.check_start <- function(x0, a, b) {
  x0 <- as.double(x0)
  if (length(x0) != length(a)) stop("x0 must have the same length as the bounds")
  pmin(pmax(x0, a), b)
}
