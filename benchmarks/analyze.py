#!/usr/bin/env python3
"""Analysis of benchmark results: performance profiles, data profiles,
FFI-overhead figure, LaTeX tables, and a summary JSON used by the paper.

Reads  benchmarks/results/results_{python,r}.csv and ffi_*.csv
Writes paper/fig/*.pdf, paper/tab/*.tex, benchmarks/results/summary.json
"""

from __future__ import annotations

import json
import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

ROOT = Path(__file__).resolve().parent.parent
RES = ROOT / "benchmarks" / "results"
FIG = ROOT / "paper" / "fig"
TAB = ROOT / "paper" / "tab"
FIG.mkdir(parents=True, exist_ok=True)
TAB.mkdir(parents=True, exist_ok=True)

# Okabe-Ito colorblind-safe palette, fixed assignment per method family.
OKABE = {
    "blue": "#0072B2", "orange": "#E69F00", "green": "#009E73",
    "vermillion": "#D55E00", "purple": "#CC79A7", "sky": "#56B4E9",
    "yellow": "#F0E442", "black": "#000000",
}

STYLE = {
    # method -> (label, color, linestyle)  solid = globalopt, dashed = external
    "globalopt_bayes1_fortran": ("BAYES1 (1989)", OKABE["blue"], "-"),
    "globalopt_exkor_fortran": ("EXKOR (1989)", OKABE["green"], "-"),
    "globalopt_glopt_fortran": ("GLOPT (1989)", OKABE["purple"], "-"),
    "globalopt_lpmin_fortran": ("LPMIN (1989)", OKABE["sky"], "-"),
    "globalopt_unt_fortran": ("UNT (1989)", OKABE["yellow"], "-"),
    "globalopt_lbayes_fortran": ("LBAYES (1989)", OKABE["orange"], "-"),
    "globalopt_mig2_fortran": ("MIG2 (1989)", "#888888", "-"),
    "scipy_dual_annealing": ("dual annealing", OKABE["vermillion"], "--"),
    "scipy_de": ("diff.\\ evolution", OKABE["orange"], "--"),
    "scipy_direct": ("DIRECT (SciPy)", OKABE["sky"], "--"),
    "scipy_shgo": ("SHGO", OKABE["yellow"], "--"),
    "nlopt_direct_l": ("DIRECT-L", OKABE["sky"], "--"),
    "nlopt_crs2": ("CRS2", OKABE["green"], "--"),
    "nlopt_mlsl": ("MLSL", OKABE["purple"], "--"),
    "nlopt_isres": ("ISRES", OKABE["yellow"], "--"),
    "cma": ("CMA-ES", OKABE["purple"], "--"),
    "gensa": ("GenSA", OKABE["vermillion"], "--"),
    "deoptim": ("DEoptim", OKABE["orange"], "--"),
    "ga": ("GA", OKABE["yellow"], "--"),
    "nloptr_direct_l": ("DIRECT-L", OKABE["sky"], "--"),
    "nloptr_crs2": ("CRS2", OKABE["green"], "--"),
    "nloptr_isres": ("ISRES", OKABE["yellow"], "--"),
    "skopt_gp": ("GP-EI (skopt)", OKABE["green"], ":"),
    "random_search": ("random search", OKABE["black"], ":"),
    "globalopt_bayes1_rust": ("BAYES1 (Rust)", OKABE["blue"], "-."),
    "globalopt_mig2_rust": ("MIG2 (Rust)", "#888888", "-."),
}

PY_PROFILE_METHODS = [
    "globalopt_bayes1_fortran", "globalopt_exkor_fortran",
    "globalopt_lbayes_fortran", "scipy_dual_annealing", "nlopt_direct_l",
    "cma", "random_search",
]
R_PROFILE_METHODS = [
    "globalopt_bayes1_fortran", "globalopt_exkor_fortran",
    "globalopt_lbayes_fortran", "gensa", "deoptim",
    "nloptr_direct_l", "random_search",
]

plt.rcParams.update({
    "font.size": 9,
    "axes.spines.top": False,
    "axes.spines.right": False,
    "axes.grid": True,
    "grid.alpha": 0.25,
    "grid.linewidth": 0.5,
    "lines.linewidth": 1.8,
    "legend.frameon": False,
    "pdf.fonttype": 42,
})

TOL_COL = {1e-1: "hit_tol1", 1e-2: "hit_tol2", 1e-4: "hit_tol4",
           1e-6: "hit_tol6"}


def load(lang: str) -> pd.DataFrame:
    df = pd.read_csv(RES / f"results_{lang}.csv")
    # expensive baselines run by separate sharded drivers (e.g. skopt_gp)
    extra = RES / f"results_{lang}_gp.csv"
    if extra.exists():
        df = pd.concat([df, pd.read_csv(extra)], ignore_index=True)
    df["error"] = df["error"].fillna("")
    return df


def perf_profile(df: pd.DataFrame, methods: list[str], tol: float,
                 budget_mult: int, ax, rmax: float = 64.0):
    """Dolan-More performance profile on evals-to-tolerance."""
    col = TOL_COL[tol]
    d = df[(df.budget == budget_mult * df.dim) & df.method.isin(methods)].copy()
    # a run "solves" the problem if it hit the tolerance within budget
    d["cost"] = d[col].where(d[col] <= d.budget, np.nan)
    key = ["problem", "dim", "instance"]
    piv = d.pivot_table(index=key, columns="method", values="cost",
                        aggfunc="min")
    best = piv.min(axis=1)
    ratios = piv.div(best, axis=0)
    # denominator: every problem instance, including those no method solved
    n_prob = len(d[key].drop_duplicates())
    taus = np.logspace(0, math.log10(rmax), 200)
    for m in methods:
        if m not in ratios:
            continue
        r = ratios[m]
        y = [(r <= t).sum() / n_prob for t in taus]
        lbl, c, ls = STYLE[m]
        ax.plot(taus, y, color=c, linestyle=ls, label=lbl)
    ax.set_xscale("log")
    ax.set_xlim(1, rmax)
    ax.set_ylim(0, 1)
    ax.set_xlabel(r"performance ratio $\tau$")
    ax.set_ylabel("fraction of problems")
    return n_prob


def data_profile(df: pd.DataFrame, methods: list[str], tol: float,
                 budget_mult: int, ax):
    """More-Wild data profile: solved fraction vs evals/(n+1)."""
    col = TOL_COL[tol]
    d = df[(df.budget == budget_mult * df.dim) & df.method.isin(methods)].copy()
    d["cost"] = d[col].where(d[col] <= d.budget, np.nan)
    d["units"] = d["cost"] / (d["dim"] + 1)
    key = ["problem", "dim", "instance"]
    piv = d.pivot_table(index=key, columns="method", values="units",
                        aggfunc="min")
    n_prob = len(d[key].drop_duplicates())
    xs = np.linspace(0, budget_mult, 200)
    for m in methods:
        if m not in piv:
            continue
        u = piv[m]
        y = [(u <= x).sum() / n_prob for x in xs]
        lbl, c, ls = STYLE[m]
        ax.plot(xs, y, color=c, linestyle=ls, label=lbl)
    ax.set_xlim(0, budget_mult)
    ax.set_ylim(0, 1)
    ax.set_xlabel(r"evaluations / $(n+1)$")
    ax.set_ylabel("fraction of problems solved")
    return n_prob


def fig_profiles(py: pd.DataFrame, r: pd.DataFrame, tol: float,
                 budget_mult: int):
    for lang, df, methods in (("python", py, PY_PROFILE_METHODS),
                              ("r", r, R_PROFILE_METHODS)):
        fig, ax = plt.subplots(figsize=(4.2, 3.1))
        n = perf_profile(df, methods, tol, budget_mult, ax)
        ax.legend(loc="upper left", fontsize=7)
        ax.set_title(
            f"{lang.capitalize()} methods, gap $\\leq {tol:g}$, "
            f"budget ${budget_mult}n$ ({n} problems)", fontsize=9)
        fig.tight_layout()
        fig.savefig(FIG / f"perf_profile_{lang}.pdf")
        plt.close(fig)

    fig, ax = plt.subplots(figsize=(4.2, 3.1))
    n = data_profile(py, PY_PROFILE_METHODS, tol, budget_mult, ax)
    ax.legend(loc="upper left", fontsize=7)
    ax.set_title(f"Python methods, gap $\\leq {tol:g}$ ({n} problems)",
                 fontsize=9)
    fig.tight_layout()
    fig.savefig(FIG / "data_profile_python.pdf")
    plt.close(fig)

    # tight-budget (25n) data profile including the GP-BO baseline, which
    # only runs on this tier (n <= 6; see run_bench.m_skopt_gp)
    if "skopt_gp" in set(py.method):
        methods_25 = ["skopt_gp", "globalopt_bayes1_fortran",
                      "globalopt_lbayes_fortran", "scipy_dual_annealing",
                      "nlopt_direct_l", "cma", "random_search"]
        d25 = py[py.dim <= 6]
        fig, ax = plt.subplots(figsize=(4.2, 3.1))
        n = data_profile(d25, methods_25, tol, 25, ax)
        ax.legend(loc="upper left", fontsize=7)
        ax.set_title(
            f"Python methods, budget $25n$, $n\\leq 6$ ({n} problems)",
            fontsize=9)
        fig.tight_layout()
        fig.savefig(FIG / "data_profile_python_25n.pdf")
        plt.close(fig)


def fig_ffi():
    before = pd.read_csv(RES / "ffi_python_before_fix.csv")
    after = pd.read_csv(RES / "ffi_python.csv")

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(5.4, 2.9),
                                   gridspec_kw={"width_ratios": [1.1, 1]})

    # Left: per-eval overhead, MIG2 (overhead-dominated), d=10
    m = after[(after.method == "mig2") & (after.dim == 10)]
    order = ["fortran+compiled-obj", "rust+native-obj",
             "fortran+py-callback", "rust+py-callback"]
    labels = ["Fortran\ncompiled", "Rust\nnative", "Fortran\nPy cb",
              "Rust\nPy cb"]
    vals = [m[m.variant == v].per_eval_us.iloc[0] for v in order]
    colors = [OKABE["blue"], OKABE["orange"], OKABE["blue"], OKABE["orange"]]
    bars = ax1.bar(labels, vals, color=colors, width=0.62)
    for b, v in zip(bars, vals):
        ax1.text(b.get_x() + b.get_width() / 2, v + 0.05, f"{v:.1f}",
                 ha="center", fontsize=7)
    ax1.set_ylabel(r"$\mu$s per evaluation")
    ax1.set_title("MIG2 call overhead ($n{=}10$)", fontsize=8.5)
    ax1.tick_params(axis="x", labelsize=6.5)

    # Right: BAYES1 runtime, before vs after the pruning repair
    dims = [2, 10, 20]
    fb = [before[(before.method == "bayes1") & (before.dim == d) &
                 (before.variant == "fortran+compiled-obj")].seconds.iloc[0]
          for d in dims]
    rb = [before[(before.method == "bayes1") & (before.dim == d) &
                 (before.variant == "rust+native-obj")].seconds.iloc[0]
          for d in dims]
    ra = [after[(after.method == "bayes1") & (after.dim == d) &
                (after.variant == "rust+native-obj")].seconds.iloc[0]
          for d in dims]
    x = np.arange(len(dims))
    w = 0.27
    ax2.bar(x - w, fb, w, label="Fortran", color=OKABE["blue"])
    ax2.bar(x, rb, w, label="Rust before", color=OKABE["vermillion"])
    ax2.bar(x + w, ra, w, label="Rust after", color=OKABE["orange"])
    ax2.set_yscale("log")
    ax2.set_xticks(x, [f"$n$={d}" for d in dims], fontsize=8)
    ax2.set_ylabel("seconds (1000 evals)")
    ax2.set_title("BAYES1 translation repair", fontsize=8.5)
    ax2.legend(fontsize=6.5)

    fig.tight_layout()
    fig.savefig(FIG / "ffi_decomposition.pdf")
    plt.close(fig)


def esc(s: str) -> str:
    return s.replace("_", r"\_")


def tab_median_gap(py: pd.DataFrame, r: pd.DataFrame):
    """Median terminal gap by method x dimension at budget 100n."""
    rows = []
    for lang, df in (("Python", py), ("R", r)):
        d = df[df.budget == 100 * df.dim]
        for m, g in d.groupby("method"):
            row = {"language": lang, "method": m}
            for dim in (2, 5, 10):
                v = g[g.dim == dim].best_f.median()
                row[f"d{dim}"] = v
            row["fixed"] = g[~g.dim.isin([2, 5, 10]) | g.problem.isin(
                ["branin", "goldstein_price", "six_hump_camel", "shekel5",
                 "shekel7", "shekel10", "hartmann3", "hartmann6"])].best_f.median()
            rows.append(row)
    t = pd.DataFrame(rows)

    def fmt(v):
        if pd.isna(v) or not np.isfinite(v):
            return "--"
        if v <= 0:
            return "0"
        return f"{v:.1e}".replace("e-0", "e-")

    lines = [
        r"\begin{tabular}{llrrrr}", r"\toprule",
        r"Ecosystem & Method & \multicolumn{3}{c}{scalable, median gap at $100n$} & classics \\",
        r" & & $n{=}2$ & $n{=}5$ & $n{=}10$ & \\",
        r"\midrule",
    ]
    for lang in ("Python", "R"):
        sub = t[t.language == lang].sort_values("d2")
        for _, row in sub.iterrows():
            name = STYLE.get(row.method, (row.method,))[0].replace("\\ ", " ")
            lines.append(
                f"{lang} & {esc(name)} & {fmt(row.d2)} & {fmt(row.d5)} & "
                f"{fmt(row.d10)} & {fmt(row.fixed)} \\\\")
        lines.append(r"\midrule" if lang == "Python" else r"\bottomrule")
    lines.append(r"\end{tabular}")
    (TAB / "median_gap.tex").write_text("\n".join(lines))
    return t


def paired_mcnemar(df: pd.DataFrame, m1: str, m2: str, tier_mult: int = 100):
    """Exact McNemar (binomial) test on paired solved/unsolved outcomes at
    gap 1e-2 within budget, over (problem, dim, instance) pairs."""
    from scipy.stats import binomtest

    d = df[df.budget == tier_mult * df.dim].copy()
    d["solved"] = d.hit_tol2.notna() & (d.hit_tol2 <= d.budget)
    key = ["problem", "dim", "instance"]
    p = d[d.method.isin([m1, m2])].pivot_table(index=key, columns="method",
                                               values="solved").dropna()
    b = int(((p[m1] == 1) & (p[m2] == 0)).sum())
    c = int(((p[m1] == 0) & (p[m2] == 1)).sum())
    pv = binomtest(b, b + c, 0.5).pvalue if b + c > 0 else 1.0
    return {"only_first": b, "only_second": c, "pairs": len(p),
            "p_value": round(float(pv), 6)}


def summarize(py: pd.DataFrame, r: pd.DataFrame, gaps: pd.DataFrame):
    out = {}
    # planned pairwise tests for the headline claims (reported with
    # Holm-Bonferroni in the paper)
    out["tests"] = {
        "py_exkor_vs_dual_annealing": paired_mcnemar(py, "globalopt_exkor_fortran", "scipy_dual_annealing"),
        "py_exkor_vs_direct_l": paired_mcnemar(py, "globalopt_exkor_fortran", "nlopt_direct_l"),
        "py_exkor_vs_cma": paired_mcnemar(py, "globalopt_exkor_fortran", "cma"),
        "py_bayes1_fortran_vs_rust": paired_mcnemar(py, "globalopt_bayes1_fortran", "globalopt_bayes1_rust"),
        "r_exkor_vs_direct_l": paired_mcnemar(r, "globalopt_exkor_fortran", "nloptr_direct_l"),
        "r_gensa_vs_exkor": paired_mcnemar(r, "gensa", "globalopt_exkor_fortran"),
    }
    for lang, df in (("python", py), ("r", r)):
        d = df[df.error == ""]
        out[f"{lang}_runs"] = int(len(df))
        out[f"{lang}_errors"] = int((df.error != "").sum())
        # success rate at 1e-2 within budget, per budget tier
        d = d.assign(solved=(d.hit_tol2.notna()) & (d.hit_tol2 <= d.budget),
                     solved4=(d.hit_tol4.notna()) & (d.hit_tol4 <= d.budget),
                     tier=np.where(d.budget == 25 * d.dim, "25n", "100n"))
        for tier in ("25n", "100n"):
            t = d[d.tier == tier]
            out[f"{lang}_solve_rate_tol2_{tier}"] = {
                m: round(float(v), 4)
                for m, v in t.groupby("method").solved.mean().items()
            }
            out[f"{lang}_median_time_{tier}"] = {
                m: round(float(v), 5)
                for m, v in t.groupby("method").time_s.median().items()
            }
        out[f"{lang}_solve_rate_tol4_100n"] = {
            m: round(float(v), 4)
            for m, v in d[d.tier == "100n"].groupby("method").solved4.mean().items()
        }
        if lang == "python":
            e = d[(d.method == "globalopt_exkor_fortran") & (d.tier == "100n")]
            out["exkor_by_problem_100n"] = {
                p: round(float(v), 3)
                for p, v in e.groupby("problem").solved.mean().items()
            }
        # mean rank on median-gap per (problem, dim, budget)
        agg = d.groupby(["problem", "dim", "budget", "method"]).best_f.median().reset_index()
        agg["rank"] = agg.groupby(["problem", "dim", "budget"]).best_f.rank(method="min")
        out[f"{lang}_mean_rank"] = {
            m: round(float(v), 2)
            for m, v in agg.groupby("method")["rank"].mean().sort_values().items()
        }
    (RES / "summary.json").write_text(json.dumps(out, indent=1))
    return out


def main():
    py = load("python")
    r = load("r")
    fig_profiles(py, r, tol=1e-2, budget_mult=100)
    fig_ffi()
    gaps = tab_median_gap(py, r)
    out = summarize(py, r, gaps)
    print("mean ranks (python):")
    for m, v in list(out["python_mean_rank"].items())[:12]:
        print(f"  {m:32s} {v}")
    print("mean ranks (r):")
    for m, v in list(out["r_mean_rank"].items())[:12]:
        print(f"  {m:32s} {v}")
    print("figures ->", FIG, "| tables ->", TAB)


if __name__ == "__main__":
    main()
