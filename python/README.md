# globalopt-py

Python package that wraps the Rust extension module `globalopt_native` and exposes the optimization APIs translated from the GlobalMinimum Fortran routines.

## References

1. J. Mockus, *Bayesian Approach to Global Optimization*, Kluwer Academic Publishers, 1989.
2. GlobalMinimum Fortran archive: https://globaloptimum.org/global/download/GlobalMinimumFortran.tar.gz

The translated routines in this package are based on the original algorithmic ideas and implementation structure in that distribution, but the numerical execution lives in Rust.

BibTeX entries are available in the repository root file `REFERENCES.bib`.

The module `globalopt.local_minimizers` exposes optional adapters for custom local refinement, including a SciPy-backed helper when SciPy is installed.

## Benchmark Extra

The module `globalopt.benchmarks` provides:

- standard benchmark functions and default problem sets
- a reproducible benchmark runner (`run_benchmarks`)
- tabular conversion helper (`benchmark_table`)

Install benchmark extras:

```bash
python3 -m pip install -e 'python[benchmark]'
```

Example:

```bash
PYTHONPATH=python python3 python/examples/benchmark_example.py
```

Release and upload instructions are in `RELEASE_PYPI.md`.

## External Benchmark Comparison

External packages used:

- `scipy` (`scipy.optimize.differential_evolution`)
- `nevergrad` (`OnePlusOne`)
- `deap` (genetic algorithm)
- `optuna` (`TPESampler`)

Because these optimizers are stochastic, comparisons should use multiple runs and summary statistics.
The benchmark runner already reports statistical summaries across seeds: `median_best`, `best_of_runs`,
`success_rate`, and `median_seconds`.

Run used for documentation:

```bash
PYTHONPATH=python python3 - <<'PY'
from pathlib import Path
from globalopt.benchmarks import run_benchmarks, write_benchmark_csv, write_benchmark_markdown, random_search_optimizer

summaries = run_benchmarks(
	dimensions=(2, 5),
	budgets=(200,),
	seeds=tuple(range(1, 6)),
	optimizers={"random_search": random_search_optimizer},
	include_scipy_de=True,
	include_nevergrad=True,
	include_deap=True,
	include_optuna=True,
)
out = Path("python/examples/output")
write_benchmark_csv(out / "benchmark_external_comparison.csv", summaries)
write_benchmark_markdown(out / "benchmark_external_comparison.md", summaries)
PY
```

Generated reports:

- `python/examples/output/benchmark_external_comparison.csv`
- `python/examples/output/benchmark_external_comparison.md`

Sample outcomes from this run (median best objective, lower is better):

- Sphere (2D): random `0.1248`, SciPy DE `0.00149`, Nevergrad `0.0`, DEAP `0.00518`, Optuna `0.00246`
- Rosenbrock (2D): random `0.1781`, SciPy DE `0.0364`, Nevergrad `0.1628`, DEAP `0.1143`, Optuna `0.0103`
- Ackley (2D): random `4.96`, SciPy DE `0.523`, Nevergrad `0.232`, DEAP `1.31`, Optuna `0.351`
