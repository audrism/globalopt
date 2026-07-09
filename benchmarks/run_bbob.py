#!/usr/bin/env python3
"""Official BBOB (COCO) suite as external validation of the custom-suite
results: 24 functions x dims {2,5,10} x instances 1-15, same recorder
protocol and budget tiers (25n, 100n) as run_bench.py, Python methods.

The optimum value of each instance is recovered by evaluating the
optimizer location that cocoex dumps via `_best_parameter('print')`, so
the recorder measures the true gap and the analysis pipeline applies
unchanged.  Writes benchmarks/results/results_bbob.csv (schema of
run_bench.py with problem = bbob function id, e.g. "bbob_f013").

Method-specific dimension/budget guards (SHGO above n=5, GP-EI above
n=6 or beyond the 25n tier) apply exactly as in run_bench.py and are
recorded as errors.
"""

from __future__ import annotations

import csv
import multiprocessing as mp
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from run_bench import METHODS, Recorder

FIELDS = (["language", "method", "problem", "dim", "instance", "budget",
           "best_f", "evals_used", "time_s", "error"]
          + [f"hit_tol{k}" for k in range(1, 9)])

_SUITE = None


def _init():
    global _SUITE
    import cocoex

    # each worker gets its own cwd: _fopt reads a fixed-name dump file
    scratch = Path(os.environ.get("BBOB_SCRATCH", "/tmp")) / f"bbob_{os.getpid()}"
    scratch.mkdir(parents=True, exist_ok=True)
    os.chdir(scratch)
    _SUITE = cocoex.Suite("bbob", "", "dimensions:2,5,10 instance_indices:1-15")


def _fopt(problem) -> float:
    problem._best_parameter("print")
    xbest = [float(v)
             for v in Path("._bbob_problem_best_parameter.txt").read_text().split()]
    return float(problem(xbest))


def one_problem(args):
    idx, budget_mult = args
    p = _SUITE[idx]
    n = p.dimension
    fopt = _fopt(p)
    lo = list(p.lower_bounds)
    hi = list(p.upper_bounds)
    budget = budget_mult * n
    func = p.id.split("_i")[0]  # bbob_f001_i02_d05 -> bbob_f001
    inst = int(p.id.split("_i")[1].split("_")[0])

    rows = []
    for mname, fn in METHODS.items():
        rec = Recorder(lambda x: p(x) - fopt, budget)
        err = ""
        t0 = time.perf_counter()
        try:
            fn(rec, lo, hi, budget, inst)
        except Exception as exc:  # noqa: BLE001
            err = f"{type(exc).__name__}: {exc}"[:200]
        dt = time.perf_counter() - t0
        row = {
            "language": "python", "method": mname, "problem": func,
            "dim": n, "instance": inst, "budget": budget,
            "best_f": rec.best_at_budget, "evals_used": rec.count,
            "time_s": round(dt, 6), "error": err,
        }
        for k in range(8):
            row[f"hit_tol{k + 1}"] = rec.hits[k] or ""
        rows.append(row)
    return rows


def main():
    import argparse

    import cocoex

    ap = argparse.ArgumentParser()
    ap.add_argument("--shard", default="0/1",
                    help="i,j,../N: run jobs whose index %% N is in {i,j,..}")
    ap.add_argument("--pool", type=int, default=20)
    args = ap.parse_args()
    classes, shard_n = args.shard.split("/")
    shard_set = {int(v) for v in classes.split(",")}
    shard_n = int(shard_n)

    suite = cocoex.Suite("bbob", "", "dimensions:2,5,10 instance_indices:1-15")
    n_prob = len(suite)
    jobs = [(i, m) for i in range(n_prob) for m in (25, 100)]
    jobs = [j for k, j in enumerate(jobs) if k % shard_n in shard_set]
    print(f"shard {args.shard}: {len(jobs)} problem-jobs, "
          f"{len(METHODS)} methods each, pool={args.pool}")

    suffix = "" if shard_n == 1 else "_shard" + "-".join(
        str(v) for v in sorted(shard_set))
    out = (Path(__file__).resolve().parent / "results"
           / f"results_bbob{suffix}.csv")
    t0 = time.time()
    done = 0
    with out.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=FIELDS)
        w.writeheader()
        with mp.Pool(args.pool, initializer=_init) as pool:
            for rows in pool.imap_unordered(one_problem, jobs, chunksize=4):
                w.writerows(rows)
                fh.flush()
                done += 1
                if done % 60 == 0:
                    print(f"[{time.time()-t0:7.0f}s] {done}/{len(jobs)}",
                          flush=True)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
