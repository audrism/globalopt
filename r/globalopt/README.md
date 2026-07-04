# globalopt R package skeleton

This directory contains a CRAN-style R package scaffold implemented directly for the R ecosystem.

Included:

- `furasn()` benchmark objective
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
write_benchmark_csv(tbl, "benchmark_results.csv")
write_benchmark_markdown(tbl, "benchmark_results.md")
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
    optimizers = benchmark_optimizers()[c("globalopt_mig2")],
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

Sample outcomes from this run (median best objective, lower is better):

- Sphere (2D): `globalopt_mig2 = 0.000134`, `DEoptim = 0.003589`, `GA = 0.000739`, `GenSA = 0.0`
- Rosenbrock (2D): `globalopt_mig2 = 0.000402`, `DEoptim = 0.0223`, `GA = 0.2147`, `GenSA ~= 4.12e-18`
- Ackley (2D): `globalopt_mig2 = 0.3509`, `DEoptim = 2.684`, `GA = 3.229`, `GenSA ~= 4.44e-16`
