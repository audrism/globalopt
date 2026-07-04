# globalopt R package skeleton

This directory contains a CRAN-style R package scaffold implemented directly for the R ecosystem.

The public API is R-first. Fortran is used only as an internal performance/reference backend in benchmarks and comparisons, not as the user-facing package interface.

## Installation

From CRAN (after release):

```r
install.packages("globalopt")
```

From GitHub (this monorepo, package in subdirectory `r/globalopt`):

```r
install.packages("remotes")
remotes::install_github("audrism/globalopt", subdir = "r/globalopt")
```

Alternative with `pak`:

```r
install.packages("pak")
pak::pak("audrism/globalopt/r/globalopt")
```

Included:

- `furasn()` benchmark objective
- translated objective set: `fush5()`, `fush7()`, `fush10()`, `fuhar3()`, `fuhar6()`, `fubran()`, `fugold()`
- `lp_tau_point()` low-discrepancy sampler
- global methods: `mig2()`, `bayes1()`, `lpmin()`, `glopt()`, `lbayes()`
- additional methods: `unt()`, `exkor()`, `extr()`, `mivar4()`, `flexi()`, `reqp()`
- analysis helpers: `anal1()`, `anal2()`
- local minimizer adapters for `stats::optim`, `optimx`, `nloptr`, and `minqa`
- benchmark helpers: `benchmark_functions()`, `benchmark_optimizers()`, `run_benchmarks()`

Benchmark example:

```r
library(globalopt)
tbl <- run_benchmarks(dimensions = c(2L, 10L), budgets = c(1000L), seeds = 1:5)
print(head(tbl))
```

Example:

```r
library(globalopt)
a <- c(-0.25, -0.125)
b <- c(0.5, 0.625)
result <- bayes1(a, b, evaluations = 200, initial_points = 20, objective = furasn,
                 local_minimizer = "optim")
print(result$best_f)
```

Release and CRAN submission instructions are in `RELEASE_CRAN.md`.

## External Benchmark Comparison

External packages used:

- `DEoptim`
- `GA`
- `GenSA`

Because these optimizers are stochastic, comparisons should use multiple runs and summary statistics.
The benchmark output reports summary metrics across seeds: `median_best`, `best_of_runs`,
`success_rate`, and `median_seconds`.

Run used for documentation:

```r
source("r/globalopt/R/globalopt.R")
source("r/globalopt/R/local_minimizers.R")
source("r/globalopt/R/benchmarks.R")

tbl <- run_benchmarks(
    dimensions = c(2L, 5L),
    budgets = c(200L),
    seeds = 1:5,
    optimizers = benchmark_optimizers(),
    include_deoptim = TRUE,
    include_ga = TRUE,
    include_gensa = TRUE
)

write_benchmark_csv(tbl, "r/globalopt/examples/output/benchmark_external_comparison.csv")
write_benchmark_markdown(tbl, "r/globalopt/examples/output/benchmark_external_comparison.md")
```

Generated reports:

- `r/globalopt/examples/output/benchmark_external_comparison.csv`
- `r/globalopt/examples/output/benchmark_external_comparison.md`
- `r/globalopt/examples/output/benchmark_narrative.md`

For interpreted benchmark results (best/worst by dimension with gap, runtime, and
memory footprint), see `docs/benchmarks/COMPARISON_NARRATIVE.md`.
