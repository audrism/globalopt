#!/usr/bin/env Rscript

`%||%` <- function(a, b) if (!is.null(a)) a else b

args_full <- commandArgs(trailingOnly = FALSE)
file_arg <- grep("^--file=", args_full, value = TRUE)
script_path <- if (length(file_arg) > 0L) sub("^--file=", "", file_arg[[1L]]) else NULL
script_dir <- if (!is.null(script_path)) dirname(normalizePath(script_path, winslash = "/")) else getwd()
repo_root <- normalizePath(file.path(script_dir, "..", "..", ".."), winslash = "/")
setwd(repo_root)

run_cmd <- function(cmd, args) {
  code <- system2(cmd, args = args)
  if (!identical(code, 0L)) {
    stop(sprintf("command failed: %s %s", cmd, paste(args, collapse = " ")))
  }
}

build_fortran_bridge <- function() {
  src_dir <- file.path(repo_root, "upstream", "GlobalMinimumFortran", "real.8")
  bridge <- file.path(src_dir, "r_furasn_bridge.f90")
  out_so <- file.path(tempdir(), "libfortran_furasn_bridge.so")

  r_bin <- file.path(R.home("bin"), "R")
  run_cmd(r_bin, c("CMD", "SHLIB", "-o", out_so, bridge))
  out_so
}

furasn_r <- function(x) {
  n <- length(x)
  if (n == 0L) return(0)
  sum((x * x) - cos(18 * x)) * (2 / n)
}

make_fortran_fn <- function(so_path) {
  dyn.load(so_path)
  function(x) {
    n <- as.integer(length(x))
    res <- .Fortran("r_furasn_bridge", x = as.double(x), n = n, res = double(1))
    as.numeric(res$res)
  }
}

benchmark_fn <- function(label, fn, x, iterations = 50000L) {
  vals <- numeric(iterations)
  t0 <- proc.time()[[3L]]
  for (i in seq_len(iterations)) {
    vals[[i]] <- fn(x)
  }
  dt <- proc.time()[[3L]] - t0
  data.frame(
    interface = label,
    iterations = as.integer(iterations),
    total_seconds = as.numeric(dt),
    mean_microseconds = as.numeric(dt) * 1e6 / iterations,
    sample_value = vals[[1L]],
    stringsAsFactors = FALSE
  )
}

set.seed(42)
x <- runif(20L, min = -1, max = 1)

fortran_so <- build_fortran_bridge()

f_fortran <- make_fortran_fn(fortran_so)

val_r <- furasn_r(x)
val_f <- f_fortran(x)

results <- do.call(rbind, list(
  benchmark_fn("r_native", furasn_r, x),
  benchmark_fn("fortran_via_dotFortran", f_fortran, x)
))

results$abs_diff_from_r <- abs(results$sample_value - val_r)

out_dir <- file.path(repo_root, "docs", "benchmarks")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

csv_path <- file.path(out_dir, "native_interface_comparison.csv")
md_path <- file.path(out_dir, "native_interface_comparison.md")

utils::write.csv(results, csv_path, row.names = FALSE)

header <- paste0("| ", paste(names(results), collapse = " | "), " |")
sep <- paste0("| ", paste(rep("---", ncol(results)), collapse = " | "), " |")
rows <- apply(results, 1, function(r) paste0("| ", paste(as.character(r), collapse = " | "), " |"))
writeLines(c(
  "# Native Interface Comparison (R vs Fortran)",
  "",
  "This benchmark compares calling the same `furasn` objective through different interfaces:",
  "",
  "- `r_native`: direct R implementation",
  "- `fortran_via_dotFortran`: direct R `.Fortran()` call into upstream Fortran",
  "",
  sprintf("Input dimension: %d", length(x)),
  sprintf("Iterations per interface: %d", results$iterations[[1L]]),
  "",
  header,
  sep,
  rows
), con = md_path)

cat("wrote", csv_path, "\n")
cat("wrote", md_path, "\n")
