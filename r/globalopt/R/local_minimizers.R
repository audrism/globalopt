#' Local minimization adapters
#'
#' Thin adapters exposing common local minimizers behind a single
#' interface, for use as the `local_minimizer` argument of [bayes1()] and
#' [lbayes()].  Each adapter takes the objective `fn`, a starting point
#' `x0` and box bounds and returns `list(par, value, convergence,
#' message)`.
#'
#' @param local_minimizer Adapter name.
#' @param fn Objective function.
#' @param x0 Numeric starting point.
#' @param lower,upper Numeric bounds.
#' @param method,control Passed through to the underlying optimizer.
#' @param ... Further arguments passed to the underlying optimizer.
#' @return `list(par, value, convergence, message)`.
#' @export
local_minimize <- function(local_minimizer = c("optim", "optimx", "nloptr", "minqa", "custom"),
                           ...) {
  local_minimizer <- match.arg(local_minimizer)
  switch(
    local_minimizer,
    optim = local_minimize_optim(...),
    optimx = local_minimize_optimx(...),
    nloptr = local_minimize_nloptr(...),
    minqa = local_minimize_minqa(...),
    custom = stop("pass a custom local minimizer function directly")
  )
}

#' @rdname local_minimize
#' @export
local_minimize_optim <- function(fn, x0, lower, upper, method = "L-BFGS-B", control = list(), ...) {
  fit <- stats::optim(
    par = x0,
    fn = fn,
    method = method,
    lower = lower,
    upper = upper,
    control = control,
    ...
  )
  list(par = fit$par, value = fit$value, convergence = fit$convergence, message = fit$message)
}

#' @rdname local_minimize
#' @export
local_minimize_optimx <- function(fn, x0, lower, upper, method = "L-BFGS-B", control = list(), ...) {
  if (!requireNamespace("optimx", quietly = TRUE)) {
    stop("Package 'optimx' is required for local_minimize_optimx()")
  }
  fit <- optimx::optimx(
    par = x0,
    fn = fn,
    method = method,
    lower = lower,
    upper = upper,
    control = control,
    ...
  )
  best_row <- fit[1, , drop = FALSE]
  list(par = as.numeric(best_row[1, seq_along(x0)]), value = as.numeric(best_row$value), convergence = 0L, message = "optimx")
}

#' @rdname local_minimize
#' @export
local_minimize_nloptr <- function(fn, x0, lower, upper, control = list(), ...) {
  if (!requireNamespace("nloptr", quietly = TRUE)) {
    stop("Package 'nloptr' is required for local_minimize_nloptr()")
  }
  fit <- nloptr::nloptr(
    x0 = x0,
    eval_f = fn,
    lb = lower,
    ub = upper,
    opts = control,
    ...
  )
  list(par = fit$solution, value = fit$objective, convergence = fit$status, message = fit$message)
}

#' @rdname local_minimize
#' @export
local_minimize_minqa <- function(fn, x0, lower, upper, control = list(), ...) {
  if (!requireNamespace("minqa", quietly = TRUE)) {
    stop("Package 'minqa' is required for local_minimize_minqa()")
  }
  fit <- minqa::bobyqa(par = x0, fn = fn, lower = lower, upper = upper, control = control, ...)
  list(par = fit$par, value = fit$fval, convergence = 0L, message = "minqa::bobyqa")
}
