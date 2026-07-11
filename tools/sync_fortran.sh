#!/usr/bin/env bash
# Sync the canonical Fortran sources into the R and Python package trees.
#
# Canonical sources:
#   - upstream/GlobalMinimumFortran/real.8/*.f  (pristine upstream, REAL*8 tree)
#   - fortran/gm_util.f   (REAL*8 conversion of upstream i1mach1.f utilities)
#   - fortran/gm_fi.f     (FI/CONSTR trampolines + ATS state access)
#   - fortran/gm_shim.c   (callback registry / trace / builtin dispatch)
#
# Both packages vendor copies because each must be self-contained
# (CRAN tarball, PyPI sdist).  Never edit the vendored copies directly.
set -euo pipefail
cd "$(dirname "$0")/.."

ALGO_FILES="anal1.f anal2.f bayes1.f exkor.f extr.f flexi.f glopt.f lbayes.f lpmin.f mig1.f mig2.f mivar4.f reqp.f unt.f"

# Patches applied to the vendored copies (documented in
# docs/DESIGN_WRAPPERS.md):
#  1. mivar4.f: the real.8 conversion renamed SRMIN/SRMAX to SDMIN/SDMAX at
#     the two places they are computed but not at the nine places they are
#     used, leaving the used names implicitly 0.0.  Rename the uses to match
#     (restores the parent-tree behavior).
#  2. exkor.f: rename its private copies of COR/ALNORM so they do not
#     collide with the identical symbols defined in extr.f when both are
#     linked into one shared library.
#  3. anal2.f: rename its private copy of FAKTKK for the same reason
#     (also defined in lpmin.f).
#  4. glopt.f: LOCOP erroneously lists the scalars SF,SL,SLL in its
#     DIMENSION statement, which modern compilers reject.
#  5. exkor.f/extr.f: in the local-parabola block, a degenerate fit on
#     the first pass jumps to label 300 and stores XA,YA before either
#     has been assigned (uninitialized locals; present in BOTH upstream
#     trees, flagged by -Wmaybe-uninitialized and observed as
#     allocation-history-dependent results).  Initialize them to the
#     current middle point (XL2,YL2), which the degenerate branch would
#     otherwise re-store idempotently.
patch_vendored() {
  local dir="$1"
  sed -i -E 's/\bSRMIN\b/SDMIN/g; s/\bSRMAX\b/SDMAX/g' "$dir/mivar4.f"
  sed -i -E 's/\bCOR\b/CORXK/g; s/\bALNORM\b/ALNMXK/g' "$dir/exkor.f"
  sed -i -E 's/\bFAKTKK\b/FAKT22/g' "$dir/anal2.f"
  sed -i 's/DIMENSION Y(M1),X(M1),S(M),SF,SL,SLL,A(M),C(M)/DIMENSION Y(M1),X(M1),S(M),A(M),C(M)/' "$dir/glopt.f"
  sed -i 's/^      N8=0$/      N8=0\n      XA=XL2\n      YA=YL2/' "$dir/exkor.f" "$dir/extr.f"
}

# R package
mkdir -p r/globalopt/src
for f in $ALGO_FILES; do
  cp upstream/GlobalMinimumFortran/real.8/"$f" r/globalopt/src/"$f"
done
patch_vendored r/globalopt/src
cp fortran/gm_util.f fortran/gm_fi.f fortran/gm_shim.c fortran/gm.h r/globalopt/src/

# Rust/Python build (build.rs compiles everything under fortran/vendor)
mkdir -p fortran/vendor
for f in $ALGO_FILES; do
  cp upstream/GlobalMinimumFortran/real.8/"$f" fortran/vendor/"$f"
done
patch_vendored fortran/vendor

echo "synced: $(echo $ALGO_FILES | wc -w) algorithm files + bridge files"
