# globalopt

Modernization of Jonas Mockus's **GlobalMinimum** (MINIMUM, 1989) Fortran
library of Bayesian and other global-optimization methods:

- an **R package** (`r/globalopt`) and a **Python package** (`python/`,
  wheel name `globalopt-py`) in which every optimizer runs the *original
  compiled Fortran* — with R/Python callback objectives, compiled built-in
  test objectives, deterministic seeding, full evaluation traces, and
  pre-call validation of all 1989 array limits;
- a **Rust crate** hosting both a faithful translation of the core methods
  and (behind the default `fortran` feature) the wrapped original;
- a **benchmark suite** comparing the 1989 methods with modern global
  optimizers in both ecosystems, and an **FFI/fidelity study** that
  resolves the "Fortran is 4x faster than Rust" observation;
- an **arXiv paper** draft (`paper/`) reporting all of the above.

Upstream source (vendored in `upstream/GlobalMinimumFortran`, X11-style
license): <https://globaloptimum.org/global/download/GlobalMinimumFortran.tar.gz>

## Layout

| Path | Contents |
| --- | --- |
| `upstream/GlobalMinimumFortran/` | pristine 1989 distribution (single-precision and `real.8` trees) |
| `fortran/` | bridge layer: `gm_util.f` (REAL*8 utility conversion), `gm_fi.f` (FI/CONSTR trampolines), `gm_shim.c` + `gm.h` (callback registry, builtins, trace, ATS state); `vendor/` = patched algorithm sources for the Rust build |
| `tools/sync_fortran.sh` | copies + patches upstream sources into `r/globalopt/src` and `fortran/vendor` (4 documented patches) |
| `r/globalopt/` | R package (Fortran compiled into the package; pure-R reference backend retained) |
| `python/` | Python package: pyo3 extension with `backend="fortran"` and `backend="rust"` |
| `src/`, `build.rs` | Rust crate: translation + Fortran backend + pyo3 bindings |
| `benchmarks/` | cross-language benchmark harness (bit-identical problem instances in R and Python), FFI-overhead experiments, analysis producing the paper's figures/tables |
| `docs/` | design notes: `DESIGN_WRAPPERS.md`, `FORTRAN_INTERFACES.md` (wrapper-ready interface spec), `FFI_FINDINGS.md` (the 4x resolution) |
| `paper/` | LaTeX sources of the arXiv paper |

## Quick start

R (requires gfortran; builds like any source package):

```r
install.packages("r/globalopt", repos = NULL, type = "source")
library(globalopt)
bayes1(c(-1, -1), c(1, 1), evaluations = 200, initial_points = 20,
       objective = "furasn")          # compiled objective, no callbacks
bayes1(c(-1, -1), c(1, 1), 200, 20, function(x) sum(x^2))  # R objective
```

Python (requires Rust toolchain + gfortran to build; wheels bundle
libgfortran):

```bash
cd python && maturin build --release && pip install ../target/wheels/*.whl
```

```python
import globalopt
r = globalopt.bayes1([-1, -1], [1, 1], 200, 20, "furasn", backend="fortran")
r2 = globalopt.bayes1([-1, -1], [1, 1], 200, 20, lambda x: sum(v*v for v in x),
                      backend="rust")
```

## Key findings (details in `docs/FFI_FINDINGS.md` and the paper)

- Foreign-function and callback overheads are microseconds per evaluation
  and nearly identical for the Fortran and Rust backends.
- The historical "4x" gap was a *translation-fidelity* defect: the Rust
  port of the BAYES1 acquisition scan had dropped two upstream pruning
  rules, costing 1.7x (n=2) to 12x (n=20). Restoring them gives bitwise
  trajectory agreement with the 1989 code and runtime parity.
- Re-implemented Shekel/Hartmann test functions in earlier R and Rust
  ports had transposed coefficient matrices — caught only by running the
  compiled original next to the rewrites.

## Reproducing the benchmark

```bash
Rscript benchmarks/run_bench.R --out benchmarks/results/results_r.csv
python benchmarks/run_bench.py --out benchmarks/results/results_python.csv
Rscript benchmarks/ffi_overhead.R
python benchmarks/ffi_overhead.py
python benchmarks/run_bbob.py    # official BBOB/COCO validation (supports --shard i,j/N --pool K)
python benchmarks/analyze.py     # writes paper/fig + paper/tab + summary.json
```

## License

MIT for the new code; the vendored GlobalMinimum sources carry their
original permissive (X11-style) notice — see
`upstream/GlobalMinimumFortran/COPYNG`.
