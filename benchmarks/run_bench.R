#!/usr/bin/env Rscript
# R-side benchmark harness; mirrors run_bench.py (same problems, instances,
# budgets, recorder semantics, CSV schema).
#
# Usage: Rscript benchmarks/run_bench.R [--quick] [--out path] [--methods a,b]

args <- commandArgs(trailingOnly = TRUE)
opt_out <- "benchmarks/results/results_r.csv"
opt_quick <- "--quick" %in% args
opt_methods <- NULL
if (length(w <- which(args == "--out"))) opt_out <- args[[w + 1L]]
if (length(w <- which(args == "--methods"))) opt_methods <- strsplit(args[[w + 1L]], ",")[[1]]

script_dir <- dirname(normalizePath(sub("^--file=", "", grep("^--file=", commandArgs(FALSE), value = TRUE)[1])))
source(file.path(script_dir, "problems.R"))

suppressMessages(library(globalopt))

TOLERANCES <- 10^-(1:8)

new_recorder <- function(fn, budget) {
  env <- new.env(parent = emptyenv())
  env$count <- 0L
  env$best <- Inf
  env$best_at_budget <- Inf
  env$hits <- rep(NA_integer_, length(TOLERANCES))
  env$wrapped <- function(x) {
    f <- as.numeric(fn(x))
    env$count <- env$count + 1L
    if (f < env$best) {
      env$best <- f
      hit <- which(is.na(env$hits) & f <= TOLERANCES)
      if (length(hit)) env$hits[hit] <- env$count
    }
    if (env$count <= budget && f < env$best_at_budget) env$best_at_budget <- f
    f
  }
  env
}

# ---------------- methods ----------------

m_random_search <- function(rec, lo, hi, budget, seed) {
  set.seed(seed)
  n <- length(lo)
  for (i in seq_len(budget)) {
    rec$wrapped(lo + runif(n) * (hi - lo))
  }
}

m_globalopt <- function(name, backend = "fortran") {
  force(name); force(backend)
  function(rec, lo, hi, budget, seed) {
    n <- length(lo)
    if (name == "bayes1") {
      b <- min(budget, 1000L)
      init <- max(2L, min(50L, b %/% 5L))
      if (backend == "fortran") {
        bayes1(lo, hi, b, init, rec$wrapped, seed = seed, trace = FALSE)
      } else {
        bayes1(lo, hi, b, init, rec$wrapped, backend = "reference")
      }
    } else if (name == "mig2") {
      if (backend == "fortran") {
        mig2(lo, hi, min(budget, 1000L), rec$wrapped, seed = seed, trace = FALSE)
      } else {
        mig2(lo, hi, min(budget, 1000L), rec$wrapped, backend = "reference")
      }
    } else if (name == "glopt") {
      glopt(lo, hi, budget, max(2L, min(150L, 5L * n)), rec$wrapped,
            seed = seed, trace = FALSE)
    } else if (name == "unt") {
      unt(lo, hi, min(budget, 500L), 0L, 5L, rec$wrapped, seed = seed,
          trace = FALSE)
    } else if (name == "lpmin") {
      analysis <- max(10L, min(300L, budget %/% 4L))
      search <- max(1L, budget - analysis)
      lpmin(lo, hi, analysis, search, rec$wrapped, seed = seed, trace = FALSE)
    } else if (name == "exkor") {
      per_coord <- max(6L, min(500L, budget %/% (2L * n)))
      exkor(lo, hi, per_coord, x0 = (lo + hi) / 2, objective = rec$wrapped,
            model_evals = 6L, cycles = 2L, first_coord = 1L, acc = 1e-4,
            trace = FALSE)
    } else if (name == "lbayes") {
      lbayes(lo, hi, max(2L, budget %/% 8L), objective = rec$wrapped,
             seed = seed, trace = FALSE)
    } else {
      stop("unknown globalopt method: ", name)
    }
  }
}

m_gensa <- function(rec, lo, hi, budget, seed) {
  set.seed(seed)
  ctrl <- list(max.call = budget, verbose = FALSE, seed = seed,
               trace.mat = FALSE)
  GenSA::GenSA(par = (lo + hi) / 2, fn = rec$wrapped, lower = lo, upper = hi,
               control = ctrl)
}

m_deoptim <- function(rec, lo, hi, budget, seed) {
  set.seed(seed)
  np <- 10L * length(lo)
  itermax <- max(1L, budget %/% np - 1L)
  ctrl <- DEoptim::DEoptim.control(NP = np, itermax = itermax, trace = FALSE)
  DEoptim::DEoptim(fn = rec$wrapped, lower = lo, upper = hi, control = ctrl)
}

m_ga <- function(rec, lo, hi, budget, seed) {
  pop <- min(50L, max(10L, 10L * length(lo)))
  maxiter <- max(1L, budget %/% pop - 1L)
  GA::ga(type = "real-valued", fitness = function(x) -rec$wrapped(x),
         lower = lo, upper = hi, popSize = pop, maxiter = maxiter,
         monitor = FALSE, seed = seed)
}

m_nloptr <- function(algorithm) {
  force(algorithm)
  function(rec, lo, hi, budget, seed) {
    set.seed(seed)
    nloptr::nloptr(
      x0 = (lo + hi) / 2,
      eval_f = rec$wrapped,
      lb = lo, ub = hi,
      opts = list(algorithm = algorithm, maxeval = budget,
                  xtol_rel = 0, ftol_rel = 0)
    )
  }
}

# Compound 1989 method (EXKOR/A2): LP-tau design -> ANAL2 eigenframe ->
# EXKOR in the rotated frame, with the influence-ratio guard.  Mirrors
# m_exkor_a2 in run_bench.py.
m_exkor_a2 <- function(rec, lo, hi, budget, seed) {
  n <- length(lo)
  best_f <- Inf; best_x <- (lo + hi) / 2
  f <- function(x) {
    v <- rec$wrapped(x)
    if (v < best_f) { best_f <<- v; best_x <<- x }
    v
  }

  m <- max(10L, min(300L, budget %/% 10L))
  pts <- t(vapply(seq_len(m), function(k) lo + lp_tau_point(k, n) * (hi - lo),
                  numeric(n)))
  vals <- apply(pts, 1, f)
  a2 <- anal2(lo, hi, pts, vals)
  rotate <- max(a2$influence_eigen) > 1.3 * max(a2$influence)
  V <- a2$eigenvectors

  half_budget <- (budget - m) %/% 2L
  per_coord <- max(6L, min(500L, half_budget %/% (2L * n)))
  exkor(lo, hi, per_coord, x0 = (lo + hi) / 2, objective = f,
        model_evals = 6L, cycles = 2L, first_coord = 1L, acc = 1e-4,
        trace = FALSE)

  xc <- best_x
  if (rotate) {
    rad <- sqrt(sum(((hi - lo) / 2)^2)) / sqrt(n)
    g <- function(y) f(pmin(pmax(xc + as.numeric(V %*% y), lo), hi))
    exkor(rep(-rad, n), rep(rad, n), per_coord, x0 = rep(0, n), objective = g,
          model_evals = 6L, cycles = 2L, first_coord = 1L, acc = 1e-4,
          trace = FALSE)
  } else {
    exkor(lo, hi, per_coord, x0 = xc, objective = f,
          model_evals = 6L, cycles = 2L, first_coord = 1L, acc = 1e-5,
          trace = FALSE)
  }
}

METHODS <- list(
  random_search = m_random_search,
  globalopt_exkor_a2 = m_exkor_a2,
  globalopt_bayes1_fortran = m_globalopt("bayes1", "fortran"),
  globalopt_mig2_fortran = m_globalopt("mig2", "fortran"),
  globalopt_glopt_fortran = m_globalopt("glopt"),
  globalopt_unt_fortran = m_globalopt("unt"),
  globalopt_lpmin_fortran = m_globalopt("lpmin"),
  globalopt_exkor_fortran = m_globalopt("exkor"),
  globalopt_lbayes_fortran = m_globalopt("lbayes"),
  # NOTE: the pure-R "reference" backends are excluded here - at ~50-90s
  # per bayes1 run they dominate wall time while the language-implementation
  # comparison is already covered by benchmarks/ffi_overhead.R.
  gensa = m_gensa,
  deoptim = m_deoptim,
  ga = m_ga,
  nloptr_direct_l = m_nloptr("NLOPT_GN_DIRECT_L"),
  nloptr_crs2 = m_nloptr("NLOPT_GN_CRS2_LM"),
  nloptr_isres = m_nloptr("NLOPT_GN_ISRES")
)

budgets_for <- function(dim) c(25L * dim, 100L * dim)

method_names <- if (is.null(opt_methods)) names(METHODS) else opt_methods
instances <- if (opt_quick) 2L else 15L

prob_specs <- default_problem_names()
if (opt_quick) prob_specs <- prob_specs[prob_specs$dim <= 4L, ]

dir.create(dirname(opt_out), recursive = TRUE, showWarnings = FALSE)
con <- file(opt_out, "w")
header <- c("language", "method", "problem", "dim", "instance", "budget",
            "best_f", "evals_used", "time_s", "error",
            paste0("hit_tol", 1:8))
writeLines(paste(header, collapse = ","), con)

t_start <- proc.time()[[3]]
n_done <- 0L
for (spec_i in seq_len(nrow(prob_specs))) {
  pname <- prob_specs$name[[spec_i]]
  dim <- prob_specs$dim[[spec_i]]
  for (inst in seq_len(instances)) {
    prob <- make_problem(pname, dim, inst)
    for (budget in budgets_for(dim)) {
      for (mname in method_names) {
        rec <- new_recorder(prob$objective, budget)
        err <- ""
        t0 <- proc.time()[[3]]
        res <- tryCatch(
          METHODS[[mname]](rec, prob$lower, prob$upper, budget, inst),
          error = function(e) {
            err <<- substr(conditionMessage(e), 1, 200)
            NULL
          }
        )
        dt <- proc.time()[[3]] - t0
        row <- c("r", mname, pname, dim, inst, budget,
                 formatC(rec$best_at_budget, digits = 17, format = "g"),
                 rec$count, round(dt, 6),
                 gsub("[,\n]", ";", err),
                 ifelse(is.na(rec$hits), "", rec$hits))
        writeLines(paste(row, collapse = ","), con)
        n_done <- n_done + 1L
      }
    }
    flush(con)
  }
  cat(sprintf("[%8.1fs] %s d=%d done (%d runs)\n",
              proc.time()[[3]] - t_start, pname, dim, n_done))
}
close(con)
cat(sprintf("wrote %s (%d runs)\n", opt_out, n_done))
