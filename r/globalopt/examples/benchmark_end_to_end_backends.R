#!/usr/bin/env Rscript

args_full <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args_full, value = TRUE)
script_path <- if (length(file_arg) > 0L) sub("^--file=", "", file_arg[[1L]]) else NULL
script_dir <- if (!is.null(script_path)) dirname(normalizePath(script_path, winslash = "/")) else getwd()
repo_root <- normalizePath(file.path(script_dir, "..", "..", ".."), winslash = "/")
setwd(repo_root)

source(file.path("r", "globalopt", "R", "globalopt.R"))

run_cmd <- function(cmd, args) {
  code <- system2(cmd, args = args)
  if (!identical(code, 0L)) {
    stop(sprintf("command failed: %s %s", cmd, paste(args, collapse = " ")))
  }
}

build_fortran_bayes1_bridge <- function() {
  src_dir <- file.path(repo_root, "upstream", "GlobalMinimumFortran", "real.8")
  bridge <- file.path(src_dir, "r_bayes1_bridge.f90")
  bayes1 <- file.path(src_dir, "bayes1.f")
  out_so <- file.path(tempdir(), "libfortran_bayes1_bridge.so")

  r_bin <- file.path(R.home("bin"), "R")
  run_cmd(r_bin, c("CMD", "SHLIB", "-o", out_so, bridge, bayes1))
  out_so
}

make_fortran_bayes1_fn <- function(so_path) {
  dyn.load(so_path)
  function(a, b, evals, initial_points) {
    n <- as.integer(length(a))
    res <- .Fortran(
      "r_bayes1_bridge",
      a = as.double(a),
      b = as.double(b),
      n = n,
      evals = as.integer(evals),
      initial_points = as.integer(initial_points),
      best_x = double(n),
      best_f = double(1)
    )
    list(best_x = as.numeric(res$best_x), best_f = as.numeric(res$best_f))
  }
}

benchmark_optimizer <- function(label, fn, a, b, evals, initial_points, runs = 50L) {
  vals <- numeric(runs)
  times <- numeric(runs)
  for (i in seq_len(runs)) {
    t0 <- proc.time()[[3L]]
    out <- fn(a, b, evals, initial_points)
    times[[i]] <- proc.time()[[3L]] - t0
    vals[[i]] <- out$best_f
  }

  data.frame(
    backend = label,
    runs = as.integer(runs),
    evaluations = as.integer(evals),
    initial_points = as.integer(initial_points),
    median_best_f = as.numeric(stats::median(vals)),
    best_of_runs = as.numeric(min(vals)),
    median_seconds = as.numeric(stats::median(times)),
    mean_seconds = as.numeric(mean(times)),
    stringsAsFactors = FALSE
  )
}

a <- c(-0.25, -0.125)
b <- c(0.5, 0.625)
evals <- 1000L
initial_points <- 20L
runs <- 20L

fortran_so <- build_fortran_bayes1_bridge()

f_f <- make_fortran_bayes1_fn(fortran_so)

results <- do.call(rbind, list(
  benchmark_optimizer("fortran_via_dotFortran_bayes1", f_f, a, b, evals, initial_points, runs)
))

out_dir <- file.path(repo_root, "docs", "benchmarks")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

csv_path <- file.path(out_dir, "end_to_end_backend_comparison.csv")
md_path <- file.path(out_dir, "end_to_end_backend_comparison.md")

utils::write.csv(results, csv_path, row.names = FALSE)

header <- paste0("| ", paste(names(results), collapse = " | "), " |")
sep <- paste0("| ", paste(rep("---", ncol(results)), collapse = " | "), " |")
rows <- apply(results, 1, function(r) paste0("| ", paste(as.character(r), collapse = " | "), " |"))
writeLines(c(
  "# End-to-End R Interface Comparison (BAYES1)",
  "",
  "This benchmark compares the R interface cost of the same full optimizer task through the Fortran foreign-function bridge:",
  "",
  "- `fortran_via_dotFortran_bayes1`: R `.Fortran()` call to Fortran `bayes1` bridge",
  "",
  sprintf("Bounds: a=(%s), b=(%s)", paste(a, collapse = ", "), paste(b, collapse = ", ")),
  sprintf("Evaluations per run: %d", evals),
  sprintf("Initial points: %d", initial_points),
  sprintf("Runs: %d", runs),
  "",
  header,
  sep,
  rows,
  "",
  "Note: this is an end-to-end interface-level measurement of the package's Fortran path."
), con = md_path)

cat("wrote", csv_path, "\n")
cat("wrote", md_path, "\n")
