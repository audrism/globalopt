#!/usr/bin/env python3
"""FFI-overhead decomposition, Python side.

Times the same algorithm (BAYES1, MIG2) through:
  - fortran backend + compiled built-in objective (no interpreter/eval)
  - fortran backend + Python callable objective
  - rust backend + native objective (builtin name -> Rust function)
  - rust backend + Python callable objective
plus raw objective-evaluation costs.  Writes benchmarks/results/ffi_python.csv.
"""

from __future__ import annotations

import csv
import statistics
import time
from pathlib import Path

import globalopt as g


def furasn_py(x):
    import math

    n = len(x)
    return sum(v * v - math.cos(18.0 * v) for v in x) * (2.0 / n)


def time_it(fn, reps):
    ts = []
    for _ in range(reps):
        t0 = time.perf_counter()
        fn()
        ts.append(time.perf_counter() - t0)
    return statistics.median(ts)


def main():
    out = Path("benchmarks/results/ffi_python.csv")
    out.parent.mkdir(parents=True, exist_ok=True)
    rows = []

    for n in (2, 10, 20):
        a = [-1.0] * n
        b = [1.0] * n
        m, lt, reps = 1000, 50, 5

        variants = {
            "fortran+compiled-obj": lambda: g.bayes1(a, b, m, lt, "furasn", backend="fortran"),
            "fortran+py-callback": lambda: g.bayes1(a, b, m, lt, furasn_py, backend="fortran"),
            "rust+native-obj": lambda: g.bayes1(a, b, m, lt, "furasn", backend="rust"),
            "rust+py-callback": lambda: g.bayes1(a, b, m, lt, furasn_py, backend="rust"),
        }
        for name, fn in variants.items():
            rows.append(dict(language="python", method="bayes1", dim=n,
                             evals=m, variant=name, seconds=time_it(fn, reps)))

        variants = {
            "fortran+compiled-obj": lambda: g.mig2(a, b, m, "furasn", backend="fortran"),
            "fortran+py-callback": lambda: g.mig2(a, b, m, furasn_py, backend="fortran"),
            "rust+native-obj": lambda: g.mig2(a, b, m, "furasn", backend="rust"),
            "rust+py-callback": lambda: g.mig2(a, b, m, furasn_py, backend="rust"),
        }
        for name, fn in variants.items():
            rows.append(dict(language="python", method="mig2", dim=n,
                             evals=m, variant=name, seconds=time_it(fn, reps)))

        # raw objective evaluation cost
        x = [0.1] * n
        k = 100000

        def loop_py():
            s = 0.0
            for _ in range(k):
                s += furasn_py(x)
            return s

        def loop_native():
            s = 0.0
            fn = g.furasn
            for _ in range(k):
                s += fn(x)
            return s

        rows.append(dict(language="python", method="objective_only", dim=n,
                         evals=k, variant="pure-python-eval",
                         seconds=time_it(loop_py, 3)))
        rows.append(dict(language="python", method="objective_only", dim=n,
                         evals=k, variant="pyo3-native-eval",
                         seconds=time_it(loop_native, 3)))

    for r in rows:
        r["per_eval_us"] = r["seconds"] / r["evals"] * 1e6

    with out.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    for r in rows:
        print(f"{r['method']:>15} d={r['dim']:<3} {r['variant']:<22} "
              f"{r['seconds']:.4f}s  {r['per_eval_us']:.2f} us/eval")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
