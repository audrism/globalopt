#!/usr/bin/env python3
"""Parallel driver for the expensive skopt_gp baseline: 25n tier, n<=6,
sharded across processes. Appends rows in run_bench.py's CSV schema to
benchmarks/results/results_python_gp.csv."""
import csv
import multiprocessing as mp
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from problems import make_problem, _SCALABLE, _FIXED
from run_bench import Recorder, m_skopt_gp


def one(job):
    pname, dim, inst = job
    prob = make_problem(pname, dim, inst)
    budget = 25 * dim
    rec = Recorder(prob.objective, budget)
    err = ""
    t0 = time.perf_counter()
    try:
        m_skopt_gp(rec, list(prob.lower), list(prob.upper), budget, inst)
    except Exception as exc:  # noqa: BLE001
        err = f"{type(exc).__name__}: {exc}"[:200]
    dt = time.perf_counter() - t0
    row = {
        "language": "python", "method": "skopt_gp", "problem": pname,
        "dim": dim, "instance": inst, "budget": budget,
        "best_f": rec.best_at_budget, "evals_used": rec.count,
        "time_s": round(dt, 6), "error": err,
    }
    for k in range(8):
        row[f"hit_tol{k + 1}"] = rec.hits[k] or ""
    return row


def main():
    jobs = []
    for name in _SCALABLE:
        for dim in (2, 5):
            jobs += [(name, dim, i) for i in range(1, 16)]
    for name, spec in _FIXED.items():
        dim = len(spec[1])
        if dim <= 6:
            jobs += [(name, dim, i) for i in range(1, 16)]
    print(f"{len(jobs)} runs")
    out = Path("benchmarks/results/results_python_gp.csv")
    fields = (["language", "method", "problem", "dim", "instance", "budget",
               "best_f", "evals_used", "time_s", "error"]
              + [f"hit_tol{k}" for k in range(1, 9)])
    t0 = time.time()
    with out.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=fields)
        w.writeheader()
        with mp.Pool(20) as pool:
            for i, row in enumerate(pool.imap_unordered(one, jobs), 1):
                w.writerow(row)
                fh.flush()
                if i % 20 == 0:
                    print(f"[{time.time()-t0:7.0f}s] {i}/{len(jobs)}", flush=True)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
