furasn <- function(x) {
  if (length(x) == 0L) {
    return(0)
  }
  sum((x * x) - cos(18 * x)) * (2 / length(x))
}

lp_tau_point <- function(c, n) {
  if (c < 1L) stop("LPTAU point index must start at 1")
  if (n < 1L || n > 20L) stop("LPTAU dimension must be in 1..=20")

  m <- 1L + as.integer(log(c) / 0.693147)
  out <- numeric(n)
  for (j in seq_len(n)) {
    s <- 0
    for (k in seq_len(m)) {
      ns <- 0L
      for (l in k:m) {
        nn <- (l - 1L) * 20L + j
        b <- .nr(nn)
        left <- floor(2 * .frac(c / (2 ^ l)))
        right <- floor(2 * .frac(b / (2 ^ (l + 1L - k))))
        ns <- ns + as.integer(left) * as.integer(right)
      }
      s <- s + .frac(0.5 * ns) / (2 ^ (k - 1L))
    }
    out[j] <- s
  }
  out
}

mig2 <- function(a, b, evaluations, objective = furasn) {
  .validate_bounds(a, b)
  if (evaluations <= 0L) stop("evaluations must be > 0")

  ats <- .ats_generator()
  points <- vector("list", evaluations)
  values <- numeric(evaluations)
  best_f <- Inf
  best_x <- rep(0, length(a))
  best_iter <- 0L

  for (k in seq_len(evaluations)) {
    z <- .sample_with_ats(ats, a, b)
    ff <- objective(z)
    points[[k]] <- z
    values[[k]] <- ff
    if (ff < best_f) {
      best_f <- ff
      best_x <- z
      best_iter <- k
    }
  }

  list(best_x = best_x, best_f = best_f, evals = evaluations, best_iter = best_iter,
       points = points, values = values)
}

bayes1 <- function(a, b, evaluations, initial_points, objective = furasn, local_minimizer = NULL) {
  .validate_bounds(a, b)
  if (evaluations <= 0L) stop("evaluations must be > 0")
  if (initial_points <= 0L || initial_points > evaluations) stop("initial_points must be in 1..=evaluations")

  planner <- .bayes_planner()
  points <- vector("list", evaluations)
  values <- numeric(evaluations)
  best_f <- Inf
  best_x <- rep(0, length(a))
  best_iter <- 0L

  for (k in seq_len(evaluations)) {
    if (k <= initial_points) {
      z <- .map_to_bounds(lp_tau_point(k, length(a)), a, b)
    } else {
      z <- planner$next_candidate(a, b, points[seq_len(k - 1L)], values[seq_len(k - 1L)], best_f)
    }

    ff <- objective(z)
    points[[k]] <- z
    values[[k]] <- ff
    if (ff < best_f) {
      best_f <- ff
      best_x <- z
      best_iter <- k
    }
  }

  result <- list(best_x = best_x, best_f = best_f, evals = evaluations, best_iter = best_iter,
                 points = points, values = values)
  .maybe_refine(result, local_minimizer, objective, a, b)
}

lpmin <- function(a, b, analysis_evals, search_evals, objective = furasn) {
  .validate_bounds(a, b)
  if (search_evals <= 0L) stop("search_evals must be > 0")

  points <- list()
  values <- numeric(0)
  best_f <- Inf
  best_x <- rep(0, length(a))
  best_iter <- 0L

  influence <- rep(0, length(a))
  if (analysis_evals > 0L) {
    analysis <- vector("list", analysis_evals)
    avals <- numeric(analysis_evals)
    for (k in seq_len(analysis_evals)) {
      z <- .map_to_bounds(lp_tau_point(k, length(a)), a, b)
      ff <- objective(z)
      analysis[[k]] <- z
      avals[[k]] <- ff
      points[[length(points) + 1L]] <- z
      values <- c(values, ff)
      if (ff < best_f) {
        best_f <- ff
        best_x <- z
        best_iter <- length(values)
      }
    }
    for (i in seq_along(a)) {
      xv <- vapply(analysis, function(p) p[[i]], numeric(1))
      influence[[i]] <- abs(stats::cor(xv, avals))
      if (!is.finite(influence[[i]])) influence[[i]] <- 0
    }
  }

  ord <- order(influence, decreasing = TRUE)
  if (length(ord) == 0L) ord <- seq_along(a)
  for (k in seq_len(search_evals)) {
    u <- lp_tau_point(k, length(a))
    z <- numeric(length(a))
    for (j in seq_along(ord)) {
      idx <- ord[[j]]
      z[[idx]] <- a[[idx]] + u[[j]] * (b[[idx]] - a[[idx]])
    }
    ff <- objective(z)
    points[[length(points) + 1L]] <- z
    values <- c(values, ff)
    if (ff < best_f) {
      best_f <- ff
      best_x <- z
      best_iter <- length(values)
    }
  }

  list(best_x = best_x, best_f = best_f, evals = length(values), best_iter = best_iter,
       points = points, values = values)
}

glopt <- function(a, b, evaluations, initial_points, local_trials, shrink, objective = furasn) {
  .validate_bounds(a, b)
  if (initial_points <= 0L || initial_points > evaluations) stop("initial_points must be in 1..=evaluations")

  seed <- bayes1(a, b, initial_points, max(1L, min(initial_points, 20L)), objective)
  best_x <- seed$best_x
  best_f <- seed$best_f
  points <- seed$points
  values <- seed$values
  ats <- .ats_generator()
  step <- 0.25

  while (length(values) < evaluations) {
    for (i in seq_len(local_trials)) {
      if (length(values) >= evaluations) break
      cand <- best_x
      for (j in seq_along(cand)) {
        delta <- (2 * .ats_next(ats) - 1) * step * (b[[j]] - a[[j]])
        cand[[j]] <- .clamp(cand[[j]] + delta, a[[j]], b[[j]])
      }
      ff <- objective(cand)
      points[[length(points) + 1L]] <- cand
      values <- c(values, ff)
      if (ff < best_f) {
        best_f <- ff
        best_x <- cand
      }
    }
    step <- max(step * shrink, 1e-5)
  }

  list(best_x = best_x, best_f = best_f, evals = length(values), best_iter = which.min(values),
       points = points, values = values)
}

unt <- function(a, b, evaluations, local_step, objective = furasn) {
  glopt(a, b, evaluations, max(2L, min(evaluations, 10L)), 2L, 1 - local_step / 2, objective)
}

exkor <- function(x0, a, b, iterations, step, shrink, objective = furasn) {
  .validate_bounds(a, b)
  x <- .project(x0, a, b)
  points <- list(x)
  values <- c(objective(x))
  best_x <- x
  best_f <- values[[1L]]
  cur_step <- step

  for (it in seq_len(iterations)) {
    improved <- FALSE
    for (i in seq_along(x)) {
      h <- cur_step * (b[[i]] - a[[i]])
      xp <- x
      xp[[i]] <- .clamp(xp[[i]] + h, a[[i]], b[[i]])
      fp <- objective(xp)
      points[[length(points) + 1L]] <- xp
      values <- c(values, fp)
      if (fp < best_f) {
        x <- xp
        best_x <- xp
        best_f <- fp
        improved <- TRUE
        next
      }
      xm <- x
      xm[[i]] <- .clamp(xm[[i]] - h, a[[i]], b[[i]])
      fm <- objective(xm)
      points[[length(points) + 1L]] <- xm
      values <- c(values, fm)
      if (fm < best_f) {
        x <- xm
        best_x <- xm
        best_f <- fm
        improved <- TRUE
      }
    }
    if (!improved) cur_step <- max(cur_step * shrink, 1e-6)
    if (cur_step <= 1e-6) break
  }

  list(best_x = best_x, best_f = best_f, evals = length(values), best_iter = which.min(values),
       points = points, values = values)
}

extr <- function(x0, a, b, evaluations, objective = furasn) {
  local <- exkor(x0, a, b, max(1L, evaluations %/% 2L), 0.3, 0.7, objective)
  global <- mig2(a, b, max(1L, evaluations - local$evals), objective)
  if (local$best_f < global$best_f) {
    global$best_x <- local$best_x
    global$best_f <- local$best_f
  }
  global$points <- c(global$points, local$points)
  global$values <- c(global$values, local$values)
  global$evals <- length(global$values)
  global$best_iter <- which.min(global$values)
  global
}

mivar4 <- function(x0, a, b, iterations, step, objective = furasn) {
  exkor(x0, a, b, iterations, step, 0.8, objective)
}

flexi <- function(x0, a, b, iterations, simplex_scale, objective = furasn) {
  obj <- function(x) objective(.project(x, a, b))
  fit <- stats::optim(x0, obj, method = "Nelder-Mead", control = list(maxit = iterations))
  x <- .project(fit$par, a, b)
  list(best_x = x, best_f = fit$value, evals = fit$counts[[1]], best_iter = fit$counts[[1]],
       points = list(x), values = c(fit$value))
}

reqp <- function(x0, a, b, iterations, penalty, penalty_growth, objective, constr) {
  mu <- penalty
  current <- .project(x0, a, b)
  all_points <- list()
  all_values <- numeric(0)
  best_x <- current
  best_f <- objective(current)

  for (it in seq_len(iterations)) {
    pen_obj <- function(x) {
      gx <- constr(x)
      viol <- pmax(gx, 0)
      objective(x) + mu * sum(viol * viol)
    }
    step_res <- mivar4(current, a, b, 1L, 0.15, pen_obj)
    current <- step_res$best_x
    val <- objective(current)
    all_points <- c(all_points, step_res$points)
    all_values <- c(all_values, step_res$values)
    if (val < best_f) {
      best_f <- val
      best_x <- current
    }
    mu <- mu * max(1, penalty_growth)
  }

  list(best_x = best_x, best_f = best_f, evals = length(all_values), best_iter = which.min(all_values),
       points = all_points, values = all_values)
}

lbayes <- function(a, b, evaluations, initial_points, local_iterations, objective = furasn, local_minimizer = NULL) {
  coarse <- bayes1(a, b, evaluations, initial_points, objective)
  local <- exkor(coarse$best_x, a, b, local_iterations, 0.2, 0.8, objective)
  result <- if (local$best_f < coarse$best_f) local else coarse
  .maybe_refine(result, local_minimizer, objective, a, b)
}

anal1 <- function(a, b, samples, objective = furasn) {
  .validate_bounds(a, b)
  pts <- vector("list", samples)
  vals <- numeric(samples)
  for (k in seq_len(samples)) {
    p <- .map_to_bounds(lp_tau_point(k, length(a)), a, b)
    pts[[k]] <- p
    vals[[k]] <- objective(p)
  }
  .analysis_from_samples(pts, vals)
}

anal2 <- function(a, b, samples, objective = furasn) {
  .validate_bounds(a, b)
  pts <- vector("list", samples)
  vals <- numeric(samples)
  for (k in seq_len(samples)) {
    u <- lp_tau_point(k, length(a))
    p <- .map_to_bounds(u * u, a, b)
    pts[[k]] <- p
    vals[[k]] <- objective(p)
  }
  res <- .analysis_from_samples(pts, vals)
  res$influence_scores <- sqrt(res$influence_scores)
  res
}

.analysis_from_samples <- function(pts, vals) {
  n <- length(pts[[1L]])
  scores <- numeric(n)
  for (i in seq_len(n)) {
    xv <- vapply(pts, function(p) p[[i]], numeric(1))
    cxy <- abs(stats::cor(xv, vals))
    scores[[i]] <- ifelse(is.finite(cxy), cxy * cxy, 0)
  }
  ord <- order(scores, decreasing = TRUE)
  list(variable_order = as.integer(ord - 1L), influence_scores = scores, samples = length(vals))
}

.maybe_refine <- function(result, local_minimizer, objective, a, b) {
  if (is.null(local_minimizer)) return(result)
  refined <- .apply_local_minimizer(local_minimizer, objective, result$best_x, a, b)
  if (is.list(refined) && !is.null(refined$value) && refined$value < result$best_f) {
    result$best_x <- refined$par
    result$best_f <- refined$value
  }
  result
}

.apply_local_minimizer <- function(local_minimizer, objective, x0, lower, upper) {
  if (is.character(local_minimizer)) {
    return(local_minimize(local_minimizer, fn = objective, x0 = x0, lower = lower, upper = upper))
  }
  if (is.function(local_minimizer)) {
    return(local_minimizer(objective, x0, lower, upper))
  }
  stop("local_minimizer must be a character adapter name or a function")
}

.validate_bounds <- function(a, b) {
  if (length(a) == 0L || length(b) == 0L) stop("bounds must not be empty")
  if (length(a) != length(b)) stop("bounds must have the same length")
  if (length(a) > 20L) stop("dimension must be <= 20")
  for (i in seq_along(a)) {
    if (a[[i]] > b[[i]]) stop(sprintf("invalid interval at index %d: a > b", i - 1L))
  }
}

.map_to_bounds <- function(unit, a, b) a + unit * (b - a)
.project <- function(x, a, b) pmin(pmax(x, a), b)
.clamp <- function(x, lo, hi) max(lo, min(hi, x))
.frac <- function(v) v - floor(v)

.ats_generator <- function() {
  env <- new.env(parent = emptyenv())
  env$state <- c(0.86515, 0.90795, 0.66155, 0.66434, 0.56558, 0.12332, 0.69186, 0.03393,
                 0.42502, 0.99224, 0.88955, 0.53758, 0.41686, 0.42163, 0.85181)
  env
}

.ats_next <- function(ats) {
  x1 <- ats$state[[1L]] + ats$state[[15L]]
  if (x1 > 1) x1 <- x1 - 1
  ats$state <- c(ats$state[-1L], x1)
  x1
}

.sample_with_ats <- function(ats, a, b) a + .ats_next(ats) * (b - a)

.bayes_planner <- function() {
  env <- new.env(parent = emptyenv())
  env$ats <- .ats_generator()
  env$x2 <- NULL
  env$next_candidate <- function(a, b, points, values, ym) {
    n <- length(a)
    l <- length(points)
    m <- max(2L, min(50L * n, max(10L * n, l * n)))
    candidates <- list(if (is.null(env$x2)) .sample_with_ats(env$ats, a, b) else env$x2)
    for (i in seq_len(m)) {
      candidates[[length(candidates) + 1L]] <- .sample_with_ats(env$ats, a, b)
    }
    scores <- vapply(candidates, function(x) .fiap1(x, points, values, ym), numeric(1))
    ord <- order(scores)
    if (length(ord) > 1L) env$x2 <- candidates[[ord[[2L]]]]
    candidates[[ord[[1L]]]]
  }
  env
}

.fiap1 <- function(x, points, values, ym) {
  if (length(points) == 0L || length(values) == 0L) return(0)
  rmax <- .Machine$double.xmax / 2
  rlrs <- 100 * .Machine$double.eps * abs(ym)
  e <- max(1e-6, rlrs)
  yk <- ym - e
  p <- max(1, values[[1L]] - yk)
  if (abs(p) < 1e-12) p <- 1
  fii <- rmax / p
  for (i in seq_along(points)) {
    obs <- points[[i]]
    pp <- values[[i]] - yk
    if (abs(pp) < 1e-12) next
    threshold <- if (pp < 1) pp * fii else if (rmax / pp > fii) pp * fii else rmax
    d <- sum((obs - x) ^ 2)
    if (d < threshold) fii <- d / pp
  }
  -fii
}

.nr <- function(i) {
  if (i <= 0L || i > 400L) stop(sprintf("LPTAU constant index out of range: %d", i))
  if (i <= 100L) return(.A1[[i]])
  if (i <= 200L) return(.A2[[i - 100L]])
  if (i <= 280L) return(.A3[[i - 200L]])
  if (i <= 340L) return(.A4[[i - 280L]])
  .A5[[i - 340L]]
}

.A1 <- c(1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,1,3,1,3,1,3,3,1,3,1,3,1,3,1,1,3,1,3,1,5,7,7,5,1,3,3,7,5,5,7,7,1,3,3,7,5,1,1,1,15,11,5,3,1,7,9,13,11,1,3,7,9,5,13,13,11,3,15,1,17,13,7,15,9,31,9,3,27,15,29,21,23,19,11,25,7,13,17)
.A2 <- c(1,51,61,43,51,59,47,57,35,53,19,51,61,37,33,7,5,11,39,63,1,85,67,49,125,25,109,43,89,69,113,47,55,97,3,37,83,103,27,13,1,255,79,147,141,89,173,43,9,25,115,97,19,97,197,101,255,29,203,65,1,257,465,439,177,321,181,225,235,103,411,233,59,353,329,463,385,111,475,451,1,771,721,1013,759,835,949,113,929,615,157,39,761,169,983,657,647,581,505,833)
.A3 <- c(1,1285,823,727,267,833,471,1601,1341,913,1725,2021,1905,375,893,1599,415,605,819,975,1,3855,4091,987,1839,4033,2515,579,3863,977,3463,2909,3379,1349,3739,347,387,2881,2821,1873,1,4369,4125,5889,6929,3913,6211,1731,1347,6197,2817,5459,8119,5121,7669,2481,7101,2677,1405,7423,1,13107,4141,6915,16241,11643,2147,11977,4417,14651,9997,2615,13207,13313,2671,5201,11469,14855,12165,5837)
.A4 <- c(1,21845,28723,16647,16565,18777,3169,7241,5087,2507,7451,13329,8965,19457,18391,3123,11699,721,709,20481,1,65535,45311,49925,17139,35225,35873,63609,12631,27109,12055,35887,9997,1033,31161,32253,15865,26903,41543,12291,1,65537,53505,116487,82207,102401,33841,81003,103445,5205,44877,97323,75591,62487,12111,78043,49173,100419,57545,86017)
.A5 <- c(1,196611,250113,83243,50979,45059,99889,15595,152645,91369,24895,83101,226659,250917,259781,63447,147489,206167,77163,12303,1,327685,276231,116529,252717,36865,247315,144417,130127,302231,508255,320901,187499,234593,36159,508757,81991,241771,357231,299025,1,983055,326411,715667,851901,299009,1032727,685617,775365,172023,574033,810643,628265,308321,232401,974837,802875,987201,378135,774207)
