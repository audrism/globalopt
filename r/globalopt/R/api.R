#' @useDynLib globalopt, .registration = TRUE
#' @keywords internal
"_PACKAGE"

# Built-in compiled objectives, in the index order of gm_shim.c.
.GM_BUILTINS <- c(
  furasn = 0L, fush5 = 1L, fush7 = 2L, fush10 = 3L,
  fuhar3 = 4L, fuhar6 = 5L, fubran = 6L, fugold = 7L
)

#' Names of the built-in compiled test objectives
#'
#' These objectives are compiled into the package (the original Fortran
#' implementations) and can be passed by name as the `objective` argument of
#' the optimizers.  When a built-in name is used with the `"fortran"` backend
#' the optimization runs entirely in compiled code, with no callback into R
#' per evaluation.
#'
#' @return A character vector of objective names.
#' @examples
#' builtin_objectives()
#' @export
builtin_objectives <- function() {
  names(.GM_BUILTINS)
}

#' Evaluate a built-in compiled objective
#'
#' @param name Name of the objective, one of [builtin_objectives()].
#' @param x Numeric vector at which to evaluate.
#' @return The objective value (numeric scalar).
#' @examples
#' eval_builtin("furasn", c(0.2, -0.1))
#' @export
eval_builtin <- function(name, x) {
  idx <- .resolve_builtin(name)
  .Call(C_gm_eval_builtin, idx, as.double(x))
}

.resolve_builtin <- function(name) {
  if (!is.character(name) || length(name) != 1L || is.na(match(name, names(.GM_BUILTINS)))) {
    stop("unknown built-in objective: ", paste(name, collapse = ", "),
         "; see builtin_objectives()")
  }
  .GM_BUILTINS[[name]]
}

# Resolve the `objective` argument for the fortran backend: an R function is
# used as a callback; a character name selects a compiled built-in.
.resolve_objective <- function(objective) {
  if (is.function(objective)) {
    return(list(obj = objective, rho = environment(objective) %||% globalenv()))
  }
  if (is.character(objective)) {
    return(list(obj = .resolve_builtin(objective), rho = globalenv()))
  }
  stop("objective must be an R function or the name of a built-in objective")
}

`%||%` <- function(a, b) if (is.null(a)) b else a

# Deterministic 15-word ATS generator state derived from an integer seed
# (Park-Miller multiplicative congruential generator).  seed = NULL keeps the
# canonical fresh-process state of the original library.
.ats_seed_state <- function(seed) {
  if (is.null(seed)) {
    return(NULL)
  }
  x <- as.double(as.integer(seed) %% 2147483647L)
  if (x <= 0) x <- x + 2147483646
  state <- numeric(15)
  for (i in seq_len(15)) {
    x <- (x * 16807) %% 2147483647
    state[[i]] <- x / 2147483647
  }
  state
}

.validate_bounds_fortran <- function(a, b, max_dim = 20L) {
  if (length(a) == 0L || length(b) == 0L) stop("bounds must not be empty")
  if (length(a) != length(b)) stop("bounds must have the same length")
  if (length(a) > max_dim) stop("dimension must be <= ", max_dim)
  if (any(!is.finite(a)) || any(!is.finite(b))) stop("bounds must be finite")
  if (any(a > b)) stop("all lower bounds must be <= upper bounds")
}

.gm_result <- function(res, method, backend) {
  res$method <- method
  res$backend <- backend
  res$best_iter <- if (!is.null(res$values)) which.min(res$values) else NA_integer_
  class(res) <- "globalopt_result"
  res
}

# Normalize a reference-backend result (points as list) to the same shape as
# the fortran backend (points as matrix).
.gm_result_ref <- function(res, method) {
  if (is.list(res$points)) {
    res$points <- do.call(rbind, res$points)
  }
  res$evals <- length(res$values)
  .gm_result(res[c("best_x", "best_f", "evals", "points", "values")], method, "reference")
}

#' @export
print.globalopt_result <- function(x, ...) {
  cat(sprintf("globalopt %s (%s backend)\n", x$method, x$backend))
  cat(sprintf("  best_f: %.10g\n", x$best_f))
  cat("  best_x:", paste(signif(x$best_x, 6), collapse = " "), "\n")
  cat(sprintf("  evaluations: %d\n", as.integer(x$evals)))
  invisible(x)
}

.maybe_refine_result <- function(result, local_minimizer, objective, a, b) {
  if (is.null(local_minimizer)) {
    return(result)
  }
  fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
  refined <- .apply_local_minimizer(local_minimizer, fn, result$best_x, a, b)
  if (is.list(refined) && !is.null(refined$value) && refined$value < result$best_f) {
    result$best_x <- refined$par
    result$best_f <- refined$value
  }
  result
}

#' Monte-Carlo global minimization (MIG1)
#'
#' Pure Monte-Carlo search over a box using the library's additive
#' lagged-Fibonacci generator, as implemented by the original Fortran
#' routine `MIG1`.
#'
#' @param a,b Numeric vectors of lower and upper bounds.
#' @param evaluations Total number of objective evaluations.
#' @param objective An R function taking a numeric vector and returning a
#'   scalar, or the name of a compiled built-in objective
#'   (see [builtin_objectives()]).
#' @param backend `"fortran"` runs the original compiled Fortran routine;
#'   `"reference"` runs the pure-R reference implementation.
#' @param trace If `TRUE`, return all sampled points and values.
#' @param seed Optional integer seed for the ATS random generator state.
#'   `NULL` uses the canonical initial state of the original library
#'   (deterministic, reproduces a fresh-process Fortran run).
#' @return An object of class `globalopt_result`: a list with elements
#'   `best_x`, `best_f`, `evals`, `points` (matrix of evaluated points, or
#'   `NULL` when `trace = FALSE`), `values`, `method`, `backend`,
#'   `best_iter`.
#' @references
#' Mockus, J. (1989). *Bayesian Approach to Global Optimization*.
#' Kluwer Academic Publishers.
#' @examples
#' r <- mig1(c(-1, -1), c(1, 1), 200, "furasn")
#' r$best_f
#' @export
mig1 <- function(a, b, evaluations, objective = "furasn",
                 backend = c("fortran", "reference"),
                 trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_mig1(a, b, as.integer(evaluations), fn), "mig1"))
  }
  .validate_bounds_fortran(a, b, 100L)
  evaluations <- as.integer(evaluations)
  if (evaluations <= 0L) stop("evaluations must be > 0")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_mig1, a, b, evaluations, o$obj, o$rho,
               isTRUE(trace), .ats_seed_state(seed))
  .gm_result(res, "mig1", "fortran")
}

#' Monte-Carlo global minimization tracking the two best points (MIG2)
#'
#' @inheritParams mig1
#' @inherit mig1 return
#' @details The Fortran backend enforces `length(a) <= 20` and
#'   `evaluations <= 1000` (limits of the original working arrays).
#' @examples
#' r <- mig2(c(-1, -1), c(1, 1), 200, "furasn")
#' r$best_f
#' @export
mig2 <- function(a, b, evaluations, objective = "furasn",
                 backend = c("fortran", "reference"),
                 trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    return(.gm_result_ref(ref_mig2(a, b, as.integer(evaluations), fn), "mig2"))
  }
  .validate_bounds_fortran(a, b)
  evaluations <- as.integer(evaluations)
  if (evaluations <= 0L || evaluations > 1000L) stop("evaluations must be in 1..1000")
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_mig2, a, b, evaluations, o$obj, o$rho,
               isTRUE(trace), .ats_seed_state(seed))
  .gm_result(res, "mig2", "fortran")
}

#' One-step Bayesian global minimization (BAYES1)
#'
#' The one-step Bayesian method of Mockus: `initial_points` evaluations at
#' LP-tau (Sobol-type) points followed by planned points chosen by
#' maximizing a one-step expected-improvement criterion over a simple
#' surrogate (routine `BAYES1` of the original library).
#'
#' @inheritParams mig1
#' @param initial_points Number of initial LP-tau points (`1 <=
#'   initial_points <= evaluations`).
#' @param local_minimizer Optional local refinement: an adapter name accepted
#'   by [local_minimize()] or a function `f(objective, x0, lower, upper)`
#'   returning `list(par, value)`.
#' @inherit mig1 return
#' @details The Fortran backend enforces `length(a) <= 20` and
#'   `evaluations <= 1000` (limits of the original working arrays).
#' @examples
#' r <- bayes1(c(-1, -1), c(1, 1), 100, 20, "furasn")
#' r$best_f
#' @export
bayes1 <- function(a, b, evaluations, initial_points, objective = "furasn",
                   local_minimizer = NULL,
                   backend = c("fortran", "reference"),
                   trace = TRUE, seed = NULL) {
  backend <- match.arg(backend)
  a <- as.double(a); b <- as.double(b)
  if (backend == "reference") {
    fn <- if (is.character(objective)) function(x) eval_builtin(objective, x) else objective
    res <- ref_bayes1(a, b, as.integer(evaluations), as.integer(initial_points), fn)
    res <- .gm_result_ref(res, "bayes1")
    return(.maybe_refine_result(res, local_minimizer, objective, a, b))
  }
  .validate_bounds_fortran(a, b)
  evaluations <- as.integer(evaluations)
  initial_points <- as.integer(initial_points)
  if (evaluations <= 0L || evaluations > 1000L) stop("evaluations must be in 1..1000")
  if (initial_points < 1L || initial_points > evaluations) {
    stop("initial_points must be in 1..evaluations")
  }
  o <- .resolve_objective(objective)
  res <- .Call(C_gm_bayes1, a, b, evaluations, initial_points, o$obj, o$rho,
               isTRUE(trace), .ats_seed_state(seed))
  res <- .gm_result(res, "bayes1", "fortran")
  .maybe_refine_result(res, local_minimizer, objective, a, b)
}

#' LP-tau (Sobol-type) low-discrepancy point
#'
#' Generates point number `c` of the LP-tau sequence in `n` dimensions in the
#' unit cube, as implemented by the original Fortran routine `LPTAU`.
#'
#' @param c Point index (>= 1).
#' @param n Dimension (1..20).
#' @param backend `"fortran"` or `"reference"`.
#' @return Numeric vector of length `n` in `[0, 1)`.
#' @examples
#' lp_tau_point(1, 2)
#' @export
lp_tau_point <- function(c, n, backend = c("fortran", "reference")) {
  backend <- match.arg(backend)
  if (c < 1) stop("LPTAU point index must start at 1")
  if (n < 1 || n > 20) stop("LPTAU dimension must be in 1..20")
  if (backend == "reference") {
    return(ref_lp_tau_point(c, n))
  }
  .Call(C_gm_lptau, as.double(c), as.integer(n))
}

#' Get or set the ATS generator state
#'
#' The original library draws random points from a 15-word additive
#' lagged-Fibonacci generator (`ATS`) whose state lives in a Fortran COMMON
#' block.  The optimizers reset it to the canonical initial state before each
#' run (or to a state derived from their `seed` argument), so these accessors
#' are mainly useful for experimentation.
#'
#' @param state Numeric vector of length 15 with values in `[0, 1)`.
#' @return `ats_state()` returns the current state.
#' @export
ats_state <- function() {
  .Call(C_gm_ats_get)
}

#' @rdname ats_state
#' @export
set_ats_state <- function(state) {
  invisible(.Call(C_gm_ats_set, as.double(state)))
}
