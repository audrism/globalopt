# Wrapper Architecture and Port-Fidelity Audit

Date: 2026-07-08

## 1. Port-fidelity audit (Rust/pure-R vs upstream Fortran)

Upstream = `upstream/GlobalMinimumFortran/real.8` (Jonas Mockus, MINIMUM, 1989;
X11-style permissive license in `COPYNG`).

| Routine | Upstream algorithm | Rust / pure-R status |
| --- | --- | --- |
| MIG1 | Pure Monte-Carlo (ATS uniform) | Faithful |
| MIG2 | Monte-Carlo keeping 2nd minimum | Faithful |
| BAYES1 | One-step Bayesian (LPTAU init + MIG2F2 planner over FIAP1 surrogate) | Faithful (same ATS, LPTAU, FIAP1 structure) |
| LBAYES | Bayesian + local descent, PAR(nu, beta) smoothing | Loose (ports: bayes1 + coordinate descent) |
| UNT | Uniform deterministic grid w/ Wiener-model uncertainty (GRAUNT/SMD/VERT) | NOT faithful (ports: glopt wrapper) |
| EXTR | 1-D global search on Wiener-process model (BP..EP interval) | NOT faithful (ports: coord-descent + Monte Carlo, n-D) |
| EXKOR | n-D coordinate optimization, each coordinate solved by EXTR-style 1-D Wiener search | NOT faithful (ports: +/- step coordinate descent) |
| MIVAR4 | Variable-metric local method w/ numeric gradients (LINE1, MIVB1) | NOT faithful (ports: exkor variant) |
| FLEXI | Flexible-tolerance Nelder-Mead simplex with equality/inequality constraints | NOT faithful (R: stats::optim NM; Rust: simple NM) |
| REQP | Recursive quadratic programming with constraints (DECOMP/SOLVE/RQP) | NOT faithful (ports: quadratic-penalty + mivar4) |
| GLOPT | Clustering global method (CLUST/EIGVAL/SEPAR, 937 lines) | NOT faithful (ports: random restart local search) |
| LPMIN | LP-tau deterministic search with FAKTKK factor analysis | Loose (ports: correlation-based influence ordering) |
| ANAL1/ANAL2 | Harmonic/variance-based variable screening (DISP/VAR/HARM) | Loose (ports: correlation screening) |
| LPTAU | Sobol-style LP-tau sequence, n<=20, tabulated numerators | Faithful |
| ATS | Additive lagged-Fibonacci generator, 15-word state | Faithful |
| FURASN/FUSH*/FUHAR*/FUBRAN/FUGOLD | Test objectives | Faithful |

Consequence for the paper and packages: only the Fortran backend runs the
*genuine* 1989 algorithms for the non-faithful rows; Rust/pure-R versions of
those are "modernized variants" and must be labeled as such.

## 2. Upstream calling conventions

- Objective is a **link-time symbol** `FI(X,N)` (REAL*8 function), not an
  argument. Constraints for FLEXI/REQP enter via `CONSTR`.
- Control parameters via `IPAR(30)` (+offset IPA) and `PAR(30)` (+IPAA);
  `IPAR(IPA+1)=IPR` printing parameter: IPR<0 disables all normal printing.
- State via COMMON blocks (=> single-threaded; serialize calls).
- Hard limits: N<=20, M<=1000 evaluations (COMMON /BS1/Y(1000)), workspace
  XN(NM) with NM>=N*M supplied by caller.
- Signatures (real.8):
  - MIG1(X,A,B,N,FM,IPAR,IPA)
  - MIG2(X,A,B,N,XN,NM,FM,IPAR,IPA)
  - BAYES1(X,A,B,N,XN,NM,FM,IPAR,IPA)
  - LBAYES(X,A,B,N,F1,IPAR,PAR,IPA,IPAA)
  - UNT(X,A,B,N,XN,NM,FM,IPAR,IPA)
  - EXTR(XM,BP,EP,YM,IPAR,PAR,IPA,IPAA)          (1-D)
  - EXKOR(X,A,B,N,FM,IPAR,PAR,IPA,IPAA)
  - MIVAR4(X,A,B,NN,B1,NM,FM,IPAR,PAR,IPA,IPAA)
  - FLEXI(Z,M1,FF,IPAR,PAR,IPA,IPAA)
  - GLOPT(XM,A,B,M,FM,IPAR,IPA)
  - LPMIN(X,A,B,N,X2,NM,FMIN,IPAR,IPA)
  - ANAL1(XP,XG,N,XX,X,NM,IPAR,IPA), ANAL2(A,B,N,XX,X,NM,IPAR,IPA)
  - REQP(X,B1,Q,A,N,FM,IPAR,PAR,IPA,IPAA)
- Utilities (ATS, LPTAU, D1MACH, I1MACH, FU* objectives) live in `i1mach1.f`.

## 3. Precision caveat

`real.8/*.f` algorithm files carry `IMPLICIT REAL*8`, but the shared
`i1mach1.f` (ATS, LPTAU, FU* objectives) still uses default REAL. The intended
build promotes default reals: compile everything with
`gfortran -fdefault-real-8` (and keep D1MACH literal constants as-is).
Verified numerically against known optima before packaging.

## 4. Wrapper design (shared by R and Python packages)

- Trampoline objective, one per package build:
  `FUNCTION FI(X,N)` calls C `gm_call_objective(x, n)` which dispatches to
  (a) a registered interpreter callback (R closure / Python callable), or
  (b) a compiled builtin objective (FURASN, ...) selected by index —
  this path never enters the interpreter and is used for FFI-overhead
  experiments and fast benchmarking.
- Same for `CONSTR` (FLEXI/REQP).
- Printing: wrappers always set IPR=-1 and validate inputs beforehand so
  upstream WRITE error paths cannot trigger.
- Not thread-safe (COMMON blocks + global callback pointer): guard with a
  lock (Python) / document single-threaded use (R).
- R package: Fortran sources vendored under `r/globalopt/src/`, built by
  R CMD SHLIB machinery; `.Call` interface via C shim (not `.Fortran`, since
  we need callback registration and result marshalling).
- Python package: same Fortran + C shim compiled into `globalopt/_fortran`
  extension (built with meson-python or a ctypes-loaded shared lib);
  Rust/pyo3 backend retained as `backend="rust"`.

## 5. The "4x Fortran vs Rust" investigation plan

Decompose observed differences into:
1. FFI entry overhead: cost of one wrapped call, trivial objective,
   R `.Fortran`/`.Call` vs Python ctypes/f2py vs pyo3.
2. Callback overhead: compiled objective in-process vs callback into
   interpreter per evaluation (dominant term, hypothesis).
3. Core algorithm speed: pure Fortran binary vs pure Rust binary, no
   interpreter involved, same eval budget (expected near-parity).
Report all three separately in the paper; the headline "4x" is expected to be
an artifact of comparing (compiled objective, Fortran) against
(interpreter callback, Rust).
