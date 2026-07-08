#!/usr/bin/env Rscript
# FFI-overhead decomposition, R side.
#
# Separates (a) the cost of one wrapped optimizer call, (b) the per-eval
# cost of calling back into R for the objective, and (c) the pure compiled
# path (built-in objective, no interpreter per evaluation), for the same
# algorithm and budget.  Writes benchmarks/results/ffi_r.csv.

suppressMessages(library(globalopt))

out_path <- "benchmarks/results/ffi_r.csv"
dir.create(dirname(out_path), recursive = TRUE, showWarnings = FALSE)

time_it <- function(expr, reps) {
  # median of `reps` timings, seconds
  t <- numeric(reps)
  for (i in seq_len(reps)) {
    t0 <- proc.time()[[3]]
    force(expr())
    t[[i]] <- proc.time()[[3]] - t0
  }
  stats::median(t)
}

rows <- list()
add <- function(...) rows[[length(rows) + 1L]] <<- data.frame(...)

for (n in c(2L, 10L, 20L)) {
  a <- rep(-1, n); b <- rep(1, n)
  m <- 1000L
  lt <- 50L
  reps <- 5L

  r_obj <- furasn  # plain R implementation

  # bayes1: R callback vs compiled builtin (identical trajectories)
  t_cb <- time_it(function() bayes1(a, b, m, lt, r_obj, trace = FALSE), reps)
  t_bi <- time_it(function() bayes1(a, b, m, lt, "furasn", trace = FALSE), reps)
  # the pure-R planner is O(evals^2 * dim); keep it tractable
  m_ref <- if (n <= 2L) m else 300L
  t_ref <- time_it(function() bayes1(a, b, m_ref, lt, r_obj, backend = "reference"), 2L) * (m / m_ref)
  add(language = "r", method = "bayes1", dim = n, evals = m,
      variant = "fortran+R-callback", seconds = t_cb)
  add(language = "r", method = "bayes1", dim = n, evals = m,
      variant = "fortran+compiled-obj", seconds = t_bi)
  add(language = "r", method = "bayes1", dim = n, evals = m,
      variant = "pure-R-reference", seconds = t_ref)

  # mig2: cheap method, overhead dominates
  t_cb <- time_it(function() mig2(a, b, m, r_obj, trace = FALSE), reps)
  t_bi <- time_it(function() mig2(a, b, m, "furasn", trace = FALSE), reps)
  t_ref <- time_it(function() mig2(a, b, m, r_obj, backend = "reference"), reps)
  add(language = "r", method = "mig2", dim = n, evals = m,
      variant = "fortran+R-callback", seconds = t_cb)
  add(language = "r", method = "mig2", dim = n, evals = m,
      variant = "fortran+compiled-obj", seconds = t_bi)
  add(language = "r", method = "mig2", dim = n, evals = m,
      variant = "pure-R-reference", seconds = t_ref)

  # raw objective evaluation cost: R function vs one .Call round trip
  x <- runif(n, -1, 1)
  k <- 20000L
  t_r <- time_it(function() { s <- 0; for (i in 1:k) s <- s + r_obj(x); s }, 3L)
  t_c <- time_it(function() { s <- 0; for (i in 1:k) s <- s + eval_builtin("furasn", x); s }, 3L)
  add(language = "r", method = "objective_only", dim = n, evals = k,
      variant = "pure-R-eval", seconds = t_r)
  add(language = "r", method = "objective_only", dim = n, evals = k,
      variant = "dotCall-eval", seconds = t_c)
}

df <- do.call(rbind, rows)
df$per_eval_us <- df$seconds / df$evals * 1e6
write.csv(df, out_path, row.names = FALSE)
print(df, digits = 4)
cat("wrote", out_path, "\n")
