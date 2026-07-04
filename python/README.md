# globalopt-py

Python package that wraps the Rust extension module `globalopt_native` and exposes the optimization APIs translated from the GlobalMinimum Fortran routines.

## References

1. J. Mockus, *Bayesian Approach to Global Optimization*, Kluwer Academic Publishers, 1989.
2. GlobalMinimum Fortran archive: https://globaloptimum.org/global/download/GlobalMinimumFortran.tar.gz

The translated routines in this package are based on the original algorithmic ideas and implementation structure in that distribution, but the numerical execution lives in Rust.

BibTeX entries are available in the repository root file `REFERENCES.bib`.

The module `globalopt.local_minimizers` exposes optional adapters for custom local refinement, including a SciPy-backed helper when SciPy is installed.

Translated objective functions exposed by the package include:

- `furasn`
- `fush5`, `fush7`, `fush10`
- `fuhar3`, `fuhar6`
- `fubran`, `fugold`

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

Globalopt methods compared in the same table:

- `globalopt_mig1`
- `globalopt_mig2`
- `globalopt_bayes1`
- `globalopt_lpmin`
- `globalopt_glopt`
- `globalopt_unt`
- `globalopt_exkor`
- `globalopt_extr`
- `globalopt_mivar4`
- `globalopt_flexi`
- `globalopt_reqp`
- `globalopt_lbayes`

Because these optimizers are stochastic, comparisons should use multiple runs and summary statistics.
The benchmark runner already reports statistical summaries across seeds: `median_best`, `best_of_runs`,
`success_rate`, and `median_seconds`.

Run used for documentation:

```bash
PYTHONPATH=python python3 - <<'PY'
from pathlib import Path
from globalopt.benchmarks import run_benchmarks, write_benchmark_csv, write_benchmark_markdown

summaries = run_benchmarks(
	dimensions=(2,),
	budgets=(200,),
	seeds=tuple(range(1, 6)),
	include_scipy_de=True,
	include_nevergrad=True,
	include_deap=True,
	include_optuna=True,
)
out = Path("python/examples/output")
write_benchmark_csv(out / "benchmark_globalopt_vs_external.csv", summaries)
write_benchmark_markdown(out / "benchmark_globalopt_vs_external.md", summaries)
PY
```

Generated reports:

- `python/examples/output/benchmark_globalopt_vs_external.csv`
- `python/examples/output/benchmark_globalopt_vs_external.md`
- `python/examples/output/benchmark_narrative.md`

For interpreted benchmark results (best/worst by dimension with gap, runtime, and
memory footprint), see `docs/benchmarks/COMPARISON_NARRATIVE.md`.
