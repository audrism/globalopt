# The "Fortran is 4x faster than Rust" claim: resolution

Date: 2026-07-08. Machine: 2x Xeon E5-2630 v2, conda-forge gcc/gfortran 
toolchain, rustc from conda-forge, Python 3.11, R 4.5.

## Claim

Early ad-hoc benchmarks in this repository suggested that wrapping the
original Fortran APIs is about four times faster than the Rust
translation. The claim conflated three separable effects, which we
measured independently (benchmarks/ffi_overhead.{py,R}, raw data in
benchmarks/results/ffi_python_before_fix.csv, ffi_python.csv, ffi_r.csv).

## Decomposition (BAYES1 and MIG2, 1000 evaluations, furasn objective)

1. **FFI / callback overhead is microseconds and nearly identical across
   backends.** MIG2 (algorithm cost ~0): Fortran-with-compiled-objective
   0.5-1.7 us/eval vs Rust-with-native-objective 0.8-2.3 us/eval (d=2..20).
   Calling back into Python for the objective adds ~1-3 us/eval to either
   backend equally; calling back into R adds ~2-4 us/eval.
2. **The real gap was algorithmic, in the translated planner.** BAYES1
   before the fix: Rust 0.32 s (d=2), 3.4 s (d=10), 13.9 s (d=20) vs
   Fortran 0.18/1.02/1.16 s in the same session -- a 1.7x/3.4x/12x
   dimension-dependent penalty
   (the historical "4x" corresponds to mid dimensions). The Rust port of
   the FIAP1 surrogate score had dropped two upstream prunings:
   - the partial-distance early exit inside the history scan
     (`IF(D.GE.P) GOTO 20` -- abandon an observation as soon as the
     accumulated squared distance reaches the threshold), whose value
     grows linearly with dimension, and
   - the second-minimum bound that aborts the scan entirely
     (`IF(FII.LE.FM) GOTO 30`, with FM maintained by MIG2F2's streaming
     two-best bookkeeping through COMMON /BAYFM/).
   The port also materialized and sorted all candidates per step
   (O(m log m) plus per-candidate allocation) where the upstream streams
   them with O(1) state.
3. **After restoring the prunings** (src/optimizers.rs, BayesPlanner /
   fiap1 rewritten in the upstream streaming form): Rust/Fortran runtime
   ratio is 1.31 / 1.12 / 1.02 at d=2/10/20 (medians of 5; the residual
   low-dimension gap is per-candidate allocation), and the evaluation
   traces agree with the original Fortran bitwise for 168/200
   evaluations (the rest differ by 1 ulp, gfortran vs Rust libm cosine).

## Related timings

- Pure-R reference implementation of BAYES1: 84x (d=10; linearly
  extrapolated from 300-eval runs, so understated) to 465x (d=2,
  measured directly) slower than the wrapped Fortran (55 s at d=2 to
  ~200 s at d=20 per 1000-eval run) -- interpreter cost on the
  O(evals^2 d) planner, not FFI.
- Raw objective evaluation (furasn): pure Python 1.0-3.0 us, pyo3 native
  0.2-0.8 us; pure R 2.1-2.8 us, .Call round trip 3.9-4.6 us.

## Conclusions for the paper

- "Language X is faster" claims about wrapped legacy code are dominated
  by *translation fidelity*, not language or FFI overhead.
- A faithful translation must preserve not only the mathematical
  algorithm but also its complexity-relevant micro-structure (early
  exits, streaming bounds); these are exactly the parts a line-by-line
  reading most easily discards as incidental control flow.
- Wrapping the original binary-identical algorithm is the only approach
  that guarantees fidelity by construction; the repaired Rust translation
  now matches it in both trajectory and speed.
- Caveat: timings taken while benchmark sweeps ran on other cores of the
  same host; medians of 5 repetitions; microsecond-scale numbers are
  indicative rather than precise.
