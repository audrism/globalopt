# Comparison Narrative

This narrative is refreshed from the latest benchmark CSV artifacts and dedicated per-optimizer memory profiling runs in this workspace.

- Difference from optimum (gap): lower is better (benchmark optima are 0).
- Computation time: lower median seconds is better.
- Memory footprint: lower peak sampled RSS from /proc is better.

## Python Interface (globalopt vs external)
### Dimension 2
- Best gap: globalopt_flexi (median gap 1.2519e-24)
- Worst gap: globalopt_mig2 (median gap 0.603456)
- Fastest: globalopt_mig2 (median time 0.000166015 s)
- Slowest: optuna_tpe (median time 0.254558 s)
- Lowest memory: globalopt_mig1 (peak RSS 15984 KB)
- Highest memory: nevergrad_ngopt (peak RSS 162032 KB)

## R Interface (globalopt vs external)
### Dimension 2
- Best gap: gensa (median gap 2.0592e-18)
- Worst gap: ga (median gap 1.27257)
- Fastest: gensa (median time 0.0005 s)
- Slowest: globalopt_bayes1 (median time 0.5145 s)
- Lowest memory: globalopt_extr (peak RSS 71712 KB)
- Highest memory: ga (peak RSS 78584 KB)

## Notes
- Gap/time rankings are computed per dimension using medians across benchmark problems from CSV outputs.
- Memory rankings come from isolated per-optimizer runs over the same benchmark problem set for that dimension and budget.
- Peak RSS is sampled from /proc while the optimizer process is running; values are suitable for relative comparisons in this environment.
