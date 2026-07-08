# globalopt (R package)

Bindings to the *GlobalMinimum* Fortran library of Bayesian and other
global-optimization methods by Jonas Mockus (1989). The original Fortran
is compiled into the package and runs by default; a pure-R reference
implementation of the core methods is available via
`backend = "reference"`.

## Installation

From this repository (requires a Fortran compiler, as for any source
package with compiled code):

```r
install.packages("remotes")
remotes::install_github("audrism/globalopt", subdir = "r/globalopt")
```

## Usage

```r
library(globalopt)

# One-step Bayesian method with a compiled built-in objective
r <- bayes1(c(-1, -1), c(1, 1), evaluations = 200, initial_points = 20,
            objective = "furasn")
r$best_f

# Any R function works as the objective
r <- bayes1(c(-1, -1), c(1, 1), 200, 20, function(x) sum((x - 0.3)^2))

# Deterministic by default (reproduces a fresh-process 1989 run);
# use `seed` for replicated stochastic runs
mig2(c(-1, -1), c(1, 1), 100, "furasn", seed = 1)
```

Methods: `bayes1()`, `mig1()`, `mig2()`, `lbayes()`, `unt()`, `glopt()`,
`extr()` (1-D), `exkor()`, `lpmin()`, `mivar4()`, `flexi()`, `reqp()`
(constrained), `anal1()`, `anal2()` (variable screening), plus
`lp_tau_point()`, `builtin_objectives()`, `eval_builtin()`.

See `vignette("globalopt")` for a tour, the method table, seeding
semantics, and the limits inherited from the 1989 working arrays
(dimension <= 20 for most methods, evaluation caps of 500-1000).
