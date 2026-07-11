#!/usr/bin/env python3
"""Python-side benchmark harness.

Every optimizer sees the objective only through a Recorder wrapper that
counts evaluations and tracks first-hit evaluation indices for the gap
tolerances (instances are constructed so the global optimum is exactly
0).  Analysis is done on the recorded curves, so different methods'
internal accounting differences do not matter; runs are truncated at the
budget during analysis.

Usage: run_bench.py [--out results_python.csv] [--instances 15]
                    [--methods m1,m2] [--quick]
"""

from __future__ import annotations

import argparse
import csv
import math
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from problems import PROBLEM_IDS, SCALABLE_DIMS, make_problem, _SCALABLE, _FIXED

TOLERANCES = [10.0**-k for k in range(1, 9)]


class Recorder:
    def __init__(self, fn, budget):
        self.fn = fn
        self.budget = budget
        self.count = 0
        self.best = math.inf
        self.best_at_budget = math.inf
        self.hits = [None] * len(TOLERANCES)  # eval index of first hit

    def __call__(self, x):
        try:
            x = list(x)
        except TypeError:
            x = [x]
        f = float(self.fn(x))
        self.count += 1
        if f < self.best:
            self.best = f
            for k, tol in enumerate(TOLERANCES):
                if self.hits[k] is None and f <= tol:
                    self.hits[k] = self.count
        if self.count <= self.budget and f < self.best_at_budget:
            self.best_at_budget = f
        return f


# ---------------- methods ----------------
# Each method: fn(recorder, lower, upper, budget, seed) -> None
# (result is read from the recorder).


def m_random_search(rec, lo, hi, budget, seed):
    import random

    rng = random.Random(seed)
    n = len(lo)
    for _ in range(budget):
        rec([lo[j] + rng.random() * (hi[j] - lo[j]) for j in range(n)])


def _ats_state(seed):
    """15-word ATS state from an integer seed (Park-Miller, matches the R
    package's .ats_seed_state)."""
    x = seed % 2147483647
    if x <= 0:
        x += 2147483646
    state = []
    for _ in range(15):
        x = (x * 16807) % 2147483647
        state.append(x / 2147483647)
    return state


def _globalopt(name, backend):
    def run(rec, lo, hi, budget, seed):
        import globalopt

        n = len(lo)
        kw = {"backend": backend}
        if backend == "fortran":
            kw["ats_state"] = _ats_state(seed)
        if name == "bayes1":
            b = min(budget, 1000)
            init = max(2, min(50, b // 5))
            globalopt.bayes1(lo, hi, b, init, rec, **kw)
        elif name == "mig2":
            globalopt.mig2(lo, hi, min(budget, 1000), rec, **kw)
        elif name == "glopt":
            globalopt.glopt(lo, hi, budget, objective=rec,
                            start_points=max(2, min(150, 5 * n)), **kw)
        elif name == "unt":
            globalopt.unt(lo, hi, min(budget, 500), objective=rec,
                          initial_points=0, max_local_minima=5, **kw)
        elif name == "lpmin":
            analysis = max(10, min(300, budget // 4))
            search = max(1, budget - analysis)
            globalopt.lpmin(lo, hi, analysis, search, rec, **kw)
        elif name == "exkor":
            per_coord = max(6, min(500, budget // (2 * n) or 6))
            x0 = [(a + b) / 2 for a, b in zip(lo, hi)]
            globalopt.exkor(x0, lo, hi, objective=rec,
                            evals_per_coord=per_coord, model_evals=6,
                            cycles=2, first_coord=1, acc=1e-4, **kw)
        elif name == "lbayes":
            globalopt.lbayes(lo, hi, objective=rec,
                             iterations=max(2, budget // 8), **kw)
        else:
            raise ValueError(name)

    return run


def m_scipy_de(rec, lo, hi, budget, seed):
    from scipy.optimize import differential_evolution

    n = len(lo)
    popsize = 15
    maxiter = max(1, budget // (popsize * n) - 1)
    differential_evolution(
        rec, bounds=list(zip(lo, hi)), maxiter=maxiter, popsize=popsize,
        seed=seed, polish=False, tol=0, init="latinhypercube",
    )


def m_scipy_da(rec, lo, hi, budget, seed):
    from scipy.optimize import dual_annealing

    dual_annealing(rec, bounds=list(zip(lo, hi)), maxfun=budget, seed=seed)


def m_scipy_direct(rec, lo, hi, budget, seed):
    from scipy.optimize import direct

    del seed  # deterministic
    direct(rec, bounds=list(zip(lo, hi)), maxfun=budget)


def m_scipy_shgo(rec, lo, hi, budget, seed):
    import numpy as np
    from scipy.optimize import shgo

    # shgo's scrambled-Sobol sampling draws from the global RNG
    np.random.seed(seed)
    if len(lo) > 5:
        # shgo's simplicial machinery grows explosively with dimension
        # (minutes and gigabytes per run at n=10); skip above n=5.
        raise RuntimeError("shgo skipped for dim > 5")
    shgo(rec, bounds=list(zip(lo, hi)),
         options={"maxfev": budget}, sampling_method="sobol")


def _nlopt(algo_name, local=False):
    def run(rec, lo, hi, budget, seed):
        import nlopt

        n = len(lo)
        algo = getattr(nlopt, algo_name)
        opt = nlopt.opt(algo, n)
        opt.set_lower_bounds(lo)
        opt.set_upper_bounds(hi)
        opt.set_min_objective(lambda x, grad: rec(x))
        opt.set_maxeval(budget)
        nlopt.srand(seed)
        if local:
            lopt = nlopt.opt(nlopt.LN_BOBYQA, n)
            lopt.set_maxeval(max(10, budget // 10))
            opt.set_local_optimizer(lopt)
        x0 = [(a + b) / 2 for a, b in zip(lo, hi)]
        try:
            opt.optimize(x0)
        except (nlopt.RoundoffLimited, nlopt.ForcedStop):
            pass

    return run


def m_cma(rec, lo, hi, budget, seed):
    import cma

    n = len(lo)
    x0 = [(a + b) / 2 for a, b in zip(lo, hi)]
    sigma = 0.25 * min(b - a for a, b in zip(lo, hi))
    opts = {
        "bounds": [list(lo), list(hi)],
        "maxfevals": budget,
        "seed": seed,
        "verbose": -9,
        "popsize": 4 + int(3 * math.log(n)),
    }
    try:
        cma.fmin(rec, x0, sigma, options=opts)
    except Exception:
        pass  # small-budget edge cases; recorder already has the data


def m_skopt_gp(rec, lo, hi, budget, seed):
    """GP-based Bayesian optimization (scikit-optimize, EI acquisition).
    Cubic-cost GP refits make budgets beyond ~150 evaluations and
    dimensions beyond ~6 impractical; run on the 25n tier, n <= 6."""
    import warnings

    if len(lo) > 6:
        raise RuntimeError("skopt_gp skipped for dim > 6")
    if budget > 25 * len(lo):
        raise RuntimeError("skopt_gp run only on the 25n budget tier")
    from skopt import gp_minimize

    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        gp_minimize(rec, list(zip(lo, hi)), n_calls=budget,
                    n_initial_points=max(5, 2 * len(lo)),
                    random_state=seed, verbose=False)


def m_exkor_a2(rec, lo, hi, budget, seed):
    """Compound 1989 method (EXKOR/A2), portfolio form: a 10% LP-tau
    design feeds ANAL2's eigenframe estimate; 45% of the budget runs
    plain EXKOR on the full box; the final 45% restarts EXKOR at the
    incumbent, in the ANAL2 eigenframe when the dominant eigen-direction
    explains markedly more influence than the best axis, otherwise in
    the original axes.  All components are the original Fortran
    routines; the recorder tracks the best point across all phases."""
    import math

    import globalopt

    del seed  # deterministic
    n = len(lo)
    best = {"f": math.inf, "x": None}

    def f(x):
        v = rec(x)
        if v < best["f"]:
            best["f"] = v
            best["x"] = list(x)
        return v

    # phase 0: LP-tau design for ANAL2 (10% of budget)
    m = max(10, min(300, budget // 10))
    pts = []
    vals = []
    for k in range(1, m + 1):
        u = globalopt.lp_tau_point(k, n)
        p = [lo[j] + u[j] * (hi[j] - lo[j]) for j in range(n)]
        pts.append(p)
        vals.append(f(p))
    a2 = globalopt.anal2(lo, hi, points=pts, values=vals, backend="fortran")
    rotate = max(a2.influence_eigen) > 1.3 * max(a2.influence)
    V = a2.eigenvectors

    # phase 1: plain EXKOR on the full box (45%)
    half_budget = (budget - m) // 2
    per_coord = max(6, min(500, half_budget // (2 * n) or 6))
    x0 = [(a + b) / 2 for a, b in zip(lo, hi)]
    globalopt.exkor(x0, lo, hi, objective=f, evals_per_coord=per_coord,
                    model_evals=6, cycles=2, first_coord=1, acc=1e-4,
                    backend="fortran")

    # phase 2: restart at the incumbent (45%), rotated iff the guard fires
    xc = best["x"]
    per_coord = max(6, min(500, half_budget // (2 * n) or 6))
    if rotate:
        rad = math.sqrt(sum(((hi[j] - lo[j]) / 2) ** 2
                            for j in range(n))) / math.sqrt(n)

        def g(y):
            x = [xc[i] + sum(y[j] * V[j][i] for j in range(n))
                 for i in range(n)]
            return f([min(max(x[i], lo[i]), hi[i]) for i in range(n)])

        globalopt.exkor([0.0] * n, [-rad] * n, [rad] * n, objective=g,
                        evals_per_coord=per_coord, model_evals=6, cycles=2,
                        first_coord=1, acc=1e-4, backend="fortran")
    else:
        globalopt.exkor(xc, lo, hi, objective=f, evals_per_coord=per_coord,
                        model_evals=6, cycles=2, first_coord=1, acc=1e-5,
                        backend="fortran")


METHODS = {
    "random_search": m_random_search,
    "globalopt_exkor_a2": m_exkor_a2,
    "skopt_gp": m_skopt_gp,
    "globalopt_bayes1_fortran": _globalopt("bayes1", "fortran"),
    "globalopt_mig2_fortran": _globalopt("mig2", "fortran"),
    "globalopt_glopt_fortran": _globalopt("glopt", "fortran"),
    "globalopt_unt_fortran": _globalopt("unt", "fortran"),
    "globalopt_lpmin_fortran": _globalopt("lpmin", "fortran"),
    "globalopt_exkor_fortran": _globalopt("exkor", "fortran"),
    "globalopt_lbayes_fortran": _globalopt("lbayes", "fortran"),
    "globalopt_bayes1_rust": _globalopt("bayes1", "rust"),
    "globalopt_mig2_rust": _globalopt("mig2", "rust"),
    "scipy_de": m_scipy_de,
    "scipy_dual_annealing": m_scipy_da,
    "scipy_direct": m_scipy_direct,
    "scipy_shgo": m_scipy_shgo,
    "nlopt_direct_l": _nlopt("GN_DIRECT_L"),
    "nlopt_crs2": _nlopt("GN_CRS2_LM"),
    "nlopt_mlsl": _nlopt("GN_MLSL_LDS", local=True),
    "nlopt_isres": _nlopt("GN_ISRES"),
    "cma": m_cma,
}


def budgets_for(dim: int) -> list[int]:
    return [25 * dim, 100 * dim]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="benchmarks/results/results_python.csv")
    ap.add_argument("--instances", type=int, default=15)
    ap.add_argument("--methods", default=None)
    ap.add_argument("--quick", action="store_true",
                    help="2 instances, dim 2 only, for smoke testing")
    args = ap.parse_args()

    method_names = list(METHODS) if not args.methods else args.methods.split(",")
    instances = 2 if args.quick else args.instances

    prob_specs = []
    for name in _SCALABLE:
        dims = (2,) if args.quick else SCALABLE_DIMS
        for dim in dims:
            prob_specs.append((name, dim))
    for name in _FIXED:
        prob_specs.append((name, len(_FIXED[name][1])))

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    fields = (
        ["language", "method", "problem", "dim", "instance", "budget",
         "best_f", "evals_used", "time_s", "error"]
        + [f"hit_tol{k}" for k in range(1, 9)]
    )

    t_start = time.time()
    n_done = 0
    with out.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=fields)
        w.writeheader()
        for pname, dim in prob_specs:
            for inst in range(1, instances + 1):
                prob = make_problem(pname, dim, inst)
                for budget in budgets_for(dim):
                    for mname in method_names:
                        fn = METHODS[mname]
                        rec = Recorder(prob.objective, budget)
                        err = ""
                        t0 = time.perf_counter()
                        try:
                            fn(rec, list(prob.lower), list(prob.upper),
                               budget, inst)
                        except Exception as exc:  # noqa: BLE001
                            err = f"{type(exc).__name__}: {exc}"[:200]
                        dt = time.perf_counter() - t0
                        row = {
                            "language": "python",
                            "method": mname,
                            "problem": pname,
                            "dim": dim,
                            "instance": inst,
                            "budget": budget,
                            "best_f": rec.best_at_budget,
                            "evals_used": rec.count,
                            "time_s": round(dt, 6),
                            "error": err,
                        }
                        for k in range(8):
                            row[f"hit_tol{k + 1}"] = rec.hits[k] or ""
                        w.writerow(row)
                        n_done += 1
                fh.flush()
            print(f"[{time.time() - t_start:8.1f}s] {pname} d={dim} done "
                  f"({n_done} runs)", flush=True)
    print(f"wrote {out} ({n_done} runs)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
