# CRAN Release Guide

This package is released as an R source tarball.

## Install From GitHub (Before CRAN Release)

Because this repository is a monorepo, install from subdirectory `r/globalopt`:

```r
install.packages("remotes")
remotes::install_github("audrism/globalopt", subdir = "r/globalopt")
```

Alternative with `pak`:

```r
install.packages("pak")
pak::pak("audrism/globalopt/r/globalopt")
```

## 1) Prerequisites

- R (current stable)
- Optional helper tools:

```r
install.packages(c("devtools", "rcmdcheck", "testthat"))
```

## 2) Run tests

From repository root:

```bash
R_LIBS_USER="$HOME/R/library" Rscript -e 'library(testthat); testthat::test_local("r/globalopt")'
```

## 3) Build source package

```bash
cd r
R CMD build globalopt
```

This creates a tarball like `globalopt_0.1.0.tar.gz`.

## 4) Run CRAN checks locally

```bash
cd r
_R_CHECK_FORCE_SUGGESTS_=false R CMD check --no-manual globalopt_0.1.0.tar.gz
```

For pre-submission parity with CRAN, also run with suggests installed and without `--no-manual`.

## 5) Submit to CRAN

- Open: https://cran.r-project.org/submit.html
- Upload `globalopt_0.1.0.tar.gz`
- Fill maintainer and submission notes.

## 6) Respond to CRAN feedback

CRAN may request changes. Iterate by:

1. Updating version in DESCRIPTION
2. Rebuilding tarball
3. Re-running checks
4. Re-submitting
