# globalopt

Rust translation of core routines from Jonas Mockus' GlobalMinimum Fortran package, plus a Python package API for the same optimization functions.

Upstream source archive downloaded from:

- https://globaloptimum.org/global/download/GlobalMinimumFortran.tar.gz

The extracted upstream code is kept in `upstream/GlobalMinimumFortran` for traceability.

## What Was Ported

This repository currently ports the following key routines and support code:

- `MIG2` (Monte Carlo global search) -> `mig2`
- `BAYES1` (one-step Bayesian-style global search loop) -> `bayes1`
- `LPTAU` LP-sequence generator -> `lp_tau_point`
- `ATS` pseudo-random generator -> `AtsGenerator`
- `FURASN` benchmark objective -> `furasn`

Both Rust and Python APIs return rich optimization results (best point, value, all sampled points, and values).

## Rust API

Main exports are in `src/lib.rs`:

- `mig2(a, b, Mig2Config, objective)`
- `bayes1(a, b, Bayes1Config, objective)`
- `lp_tau_point(c, n)`
- `AtsGenerator`
- `furasn(x)`

### Rust examples

- `examples/mig2_furasn.rs`
- `examples/bayes1_furasn.rs`

Run examples:

```bash
cargo run --example mig2_furasn
cargo run --example bayes1_furasn
```

## Python Package API

Python package lives in `python/globalopt` and is a thin wrapper around the Rust extension module `globalopt_native`.

Functions include:

- `globalopt.mig1(...)`, `globalopt.mig2(...)`, `globalopt.bayes1(...)`
- `globalopt.lpmin(...)`, `globalopt.glopt(...)`, `globalopt.lbayes(...)`
- `globalopt.unt(...)`, `globalopt.exkor(...)`, `globalopt.extr(...)`
- `globalopt.mivar4(...)`, `globalopt.flexi(...)`, `globalopt.reqp(...)`
- `globalopt.anal1(...)`, `globalopt.anal2(...)`
- `globalopt.lp_tau_point(c, n)`, `globalopt.furasn(x)`

Examples:

- `python/examples/mig2_example.py`
- `python/examples/bayes1_example.py`

Run examples directly:

```bash
PYTHONPATH=python python3 python/examples/mig2_example.py
PYTHONPATH=python python3 python/examples/bayes1_example.py
```

Install package locally:

```bash
python3 -m pip install -e python
```

The Python API also exposes local minimizer adapters in `globalopt.local_minimizers`, and `bayes1(...)` can optionally refine its best point with a custom local minimizer callable.

Benchmark extra is provided in `globalopt.benchmarks` (problem set + runner + summary table).

External benchmark comparisons are included with statistical summaries across multiple seeds
(`median_best`, `best_of_runs`, `success_rate`, `median_seconds`) and example reports in:

- `python/examples/output/benchmark_external_comparison.csv`
- `r/globalopt/examples/output/benchmark_external_comparison.csv`

## R Package Sketch

A separate CRAN-style package skeleton lives in `r/globalopt` for the R ecosystem, with no runtime dependency on Python.

It provides standalone R implementations for:

- `mig2()`, `bayes1()`, `lpmin()`, `glopt()`, `lbayes()`
- `unt()`, `exkor()`, `extr()`, `mivar4()`, `flexi()`, `reqp()`
- `anal1()`, `anal2()`, `lp_tau_point()`, `furasn()`

It also includes local minimizer adapters for `stats::optim`, `optimx`, `nloptr`, and `minqa`.

Benchmark extra is provided via `benchmark_functions()`, `benchmark_optimizers()`, and `run_benchmarks()`.

## Notes

- The original Fortran package contains many more routines (`GLOPT`, `LPMIN`, constraints handling, etc.).
- This initial port focuses on the core search routines that were easiest to validate quickly and expose consistently in Rust and Python.

## References

Original materials cited by the upstream Fortran distribution:

1. J. Mockus, *Bayesian Approach to Global Optimization*, Kluwer Academic Publishers, Dordrecht-Boston-London, 1989. ISBN 0-7923-0115-3.
2. GlobalMinimum Fortran source distribution README and routines by Jonas Mockus (archived from https://globaloptimum.org/global/download/GlobalMinimumFortran.tar.gz, extracted in `upstream/GlobalMinimumFortran`).

Attribution note:

- This repository is a Rust/Python translation of selected algorithms inspired by and traced to the original GlobalMinimum Fortran implementation and documentation.
- BibTeX entries are provided in `REFERENCES.bib`.