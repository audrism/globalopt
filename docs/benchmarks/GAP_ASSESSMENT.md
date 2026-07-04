# Gap Assessment

Date: 2026-07-04

## Scope

This assessment compares the upstream GlobalMinimum Fortran routines in `upstream/GlobalMinimumFortran` against current implementations and API exposure in:

- Rust core (`src/optimizers.rs`, `src/lptau.rs`, `src/benchmarks.rs`)
- Python API (`python/globalopt/api.py`)
- R API (`r/globalopt/R/globalopt.R` and `r/globalopt/NAMESPACE`)

## Primary Optimizer/Analysis Routines

| Upstream routine | Rust core | Python API | R API | Status |
| --- | --- | --- | --- | --- |
| MIG1 | Yes | Yes | No | Partial across interfaces (missing in R) |
| MIG2 | Yes | Yes | Yes | Ported |
| BAYES1 | Yes | Yes | Yes | Ported |
| LPMIN | Yes | Yes | Yes | Ported |
| GLOPT | Yes | Yes | Yes | Ported |
| UNT | Yes | Yes | Yes | Ported |
| EXKOR | Yes | Yes | Yes | Ported |
| EXTR | Yes | Yes | Yes | Ported |
| MIVAR4 | Yes | Yes | Yes | Ported |
| FLEXI | Yes | Yes | Yes | Ported |
| REQP | Yes | Yes | Yes | Ported |
| LBAYES | Yes | Yes | Yes | Ported |
| ANAL1 | Yes | Yes | Yes | Ported |
| ANAL2 | Yes | Yes | Yes | Ported |

## Support/Utility Routines From Upstream

| Upstream routine family | Rust core | Python API | R API | Notes |
| --- | --- | --- | --- | --- |
| LPTAU | Yes (`lp_tau_point`) | Yes | Yes | Ported |
| ATS RNG | Yes (`AtsGenerator`) | Not explicitly exposed as ATS API | Not exposed | Partial |
| FURASN benchmark function | Yes | Yes | Yes | Ported |
| FUSH5/FUSH7/FUSH10 | Yes | Yes | Yes | Ported |
| FUHAR3/FUHAR6 | Yes | Yes | Yes | Ported |
| FUBRAN/FUGOLD | Yes | Yes | Yes | Ported |
| I1MACH variants | No direct equivalent | No | No | Not ported |

## Internal Subroutine Parity

Many internal helper subroutines in upstream files (for example clustering/eigendecomposition helpers in `glopt.f`, line/search helpers in `mivar4.f`, and decomposition helpers in `reqp.f`) are not intended as public entry points.

Current code provides algorithm-level reimplementations, not one-to-one source translation of every internal Fortran subroutine.

## Interface and Benchmark Gaps

1. R API does not currently expose MIG1.
2. One-to-one compatibility with upstream numerical behavior for all routines has not been formally verified against the original Fortran outputs.

## Recommended Next Steps

1. Add `mig1` to R API for interface parity.
2. Add regression tests comparing selected outputs against upstream Fortran for shared routines.
3. Add a standardized benchmark profile matrix (for example dimensions 2/10/30 and budgets 1k/10k) and store machine-readable metadata for each run.
