# globalopt-py

Python package that wraps the native extension module `globalopt_native` and exposes the GlobalMinimum optimization routines through two backends:

- `backend="rust"` — the Rust translation of the routines (default for the pre-0.2 signatures).
- `backend="fortran"` — the **original 1989 Fortran routines** (Mockus, GlobalMinimum), compiled into the extension. This is the genuine algorithm for every method.

## Backends

Fidelity of the Rust translation relative to the upstream Fortran (from the port-fidelity audit in `docs/DESIGN_WRAPPERS.md`). For every method, `backend="fortran"` runs the original algorithm; where the Rust column says "modernized variant", results from the two backends are *not* comparable.

| Method | `backend="fortran"` | `backend="rust"` |
| --- | --- | --- |
| `mig1` | original | faithful translation |
| `mig2` | original | faithful translation |
| `bayes1` | original | faithful translation |
| `lp_tau_point` | original | faithful translation |
| `lbayes` | original | loose port (bayes1 + coordinate descent) |
| `lpmin` | original | loose port (correlation-based ordering) |
| `unt` | original | modernized variant |
| `extr` | original (1-D Wiener model) | modernized variant |
| `exkor` | original | modernized variant (+/- step coordinate descent) |
| `mivar4` | original | modernized variant (BFGS-style) |
| `flexi` | original (constrained flexible tolerance) | modernized variant (simple Nelder-Mead) |
| `reqp` | original (recursive QP, constrained) | modernized variant (quadratic penalty) |
| `glopt` | original (clustering method) | modernized variant (random restarts) |
| `anal1` / `anal2` | not wrapped | loose port (correlation screening) |

Backend usage notes:

- `objective` may be a Python callable or, with `backend="fortran"`, the name of a compiled builtin (`"furasn"`, `"fush5"`, `"fush7"`, `"fush10"`, `"fuhar3"`, `"fuhar6"`, `"fubran"`, `"fugold"`); builtins skip Python-callback overhead entirely.
- `ats_state` (a sequence of 15 floats) seeds the ATS random generator for reproducible runs; omitted, each call starts from the canonical fresh-process state.
- The Fortran backend enforces the original limits (e.g. dimension <= 20 for most methods, evaluation budgets bounded by the /BS1/ workspace) and raises `ValueError` before any Fortran code runs.
- `flexi` and `reqp` with `backend="fortran"` are the genuinely *constrained* methods: pass `n_eq`/`n_ineq` and a `constraints(x)` callable returning equality values (=0 feasible) followed by inequality values (>=0 feasible). They take no bounds.
- `extr` **changed signature in 0.2.0**: it is one-dimensional, `extr(bp, ep, evaluations, ...)`, as in the original library, and defaults to `backend="fortran"`.
- Bounds/points accept any numeric sequence, including numpy arrays.

```python
import globalopt as go

# Original Fortran BAYES1, compiled builtin objective (no Python callbacks):
res = go.bayes1([-0.25, -0.125], [0.5, 0.625], 200, 20,
                objective="furasn", backend="fortran")

# Same but with a Python callable:
res = go.bayes1([-0.25, -0.125], [0.5, 0.625], 200, 20,
                objective=lambda x: sum(v * v for v in x), backend="fortran")
```

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
