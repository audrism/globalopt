# Replication guide (TOMS RCR reviewers start here)

Every number and figure in the paper is produced by a script in this
repository. Three replication tiers, from minutes to hours:

## Tier 1: regenerate the paper's figures, table, and statistics (~1 min)

The raw benchmark CSVs are committed under `benchmarks/results/`.

```bash
python benchmarks/analyze.py
```

writes `paper/fig/*.pdf`, `paper/tab/median_gap.tex`, and
`benchmarks/results/summary.json`. Every quantitative claim in
Section 6 of the paper appears in `summary.json` (solve rates per
budget tier, median times, exact McNemar tests with discordant counts
and p-values). The paper builds with `tectonic paper/main.tex` (arXiv
version) and `tectonic paper/main_toms.tex` (TOMS version).

## Tier 2: verify the packages against the 1989 binary (~10 min)

Toolchain: conda-forge gfortran/rust/R/python (a micromamba environment
spec is in the README; any recent versions work).

```bash
cargo test --release                 # 9 tests incl. trace-level pins
cd python && maturin build --release && pip install ../target/wheels/*.whl
python -m pytest python/tests -q     # 14 tests
R CMD INSTALL r/globalopt
Rscript -e 'testthat::test_local("r/globalopt")'   # 55 assertions
```

The test suites pin LP-tau points, built-in objective values, and full
BAYES1/MIG2 runs to reference values obtained from a standalone C
driver linked directly against the compiled 1989 library
(docs/FORTRAN_INTERFACES.md documents their provenance). The
translation-fidelity claims of Section 4 are asserted here: the Rust
BAYES1 must agree with the wrapped Fortran bitwise on `best_f`.

## Tier 3: regenerate the benchmark data (hours)

```bash
# custom suite, Python (~30 min single host; excludes the GP tier)
python benchmarks/run_bench.py --out benchmarks/results/results_python.csv \
    --methods $(python -c "import sys; sys.path.insert(0,'benchmarks'); \
from run_bench import METHODS; print(','.join(k for k in METHODS if k!='skopt_gp'))")

# custom suite, R (~40 min)
Rscript benchmarks/run_bench.R --out benchmarks/results/results_r.csv

# GP-EI tier (~30 min on 20 cores)
python benchmarks/run_gp_parallel.py

# BBOB validation (~3 h on one 20-core host; supports multi-host sharding)
benchmarks/launch_bbob_shard.sh 0/1 20

# FFI overhead decomposition (~10 min)
python benchmarks/ffi_overhead.py && Rscript benchmarks/ffi_overhead.R
```

Then rerun Tier 1. Determinism notes: the wrapped 1989 methods, EXKOR/A2,
LP-tau designs, and problem instances are fully deterministic (seeded
ATS states; Park-Miller instance shifts verified bit-identical between
the R and Python harnesses); external stochastic solvers are seeded per
instance. Wall-clock timings will differ on other hardware; solve
rates, hit indices, and discordance counts should reproduce exactly for
the deterministic methods and statistically for the seeded ones
(scipy/NLopt/CMA results can shift across library versions; the paper's
CSVs record scipy 1.13, nlopt 2.x, cma 4.x from conda-forge).

Cross-language identity check (the paper's strongest internal control):
after Tier 3, EXKOR and EXKOR/A2 rows in `results_python.csv` and
`results_r.csv` must match `best_f` bitwise on every (problem, dim,
instance, budget).
