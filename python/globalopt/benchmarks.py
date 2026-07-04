from __future__ import annotations

from dataclasses import dataclass
import csv
import math
from pathlib import Path
import time
from statistics import median
from typing import Any, Callable, Sequence

try:
    from .api import bayes1, exkor, extr, flexi, glopt, lbayes, lpmin, mig1, mig2, mivar4, reqp, unt
except Exception:  # pragma: no cover - allows benchmarking external methods when native module is unavailable
    bayes1 = None  # type: ignore[assignment]
    exkor = None  # type: ignore[assignment]
    extr = None  # type: ignore[assignment]
    flexi = None  # type: ignore[assignment]
    glopt = None  # type: ignore[assignment]
    lbayes = None  # type: ignore[assignment]
    lpmin = None  # type: ignore[assignment]
    mig1 = None  # type: ignore[assignment]
    mig2 = None  # type: ignore[assignment]
    mivar4 = None  # type: ignore[assignment]
    reqp = None  # type: ignore[assignment]
    unt = None  # type: ignore[assignment]

Objective = Callable[[Sequence[float]], float]
Optimizer = Callable[[Sequence[float], Sequence[float], int, Objective, int], float]


@dataclass
class BenchmarkProblem:
    name: str
    objective: Objective
    lower: list[float]
    upper: list[float]
    optimum: float = 0.0


@dataclass
class BenchmarkSummary:
    optimizer: str
    problem: str
    dimension: int
    budget: int
    runs: int
    median_best: float
    best_of_runs: float
    success_rate: float
    median_seconds: float


def sphere(x: Sequence[float]) -> float:
    return sum(v * v for v in x)


def rosenbrock(x: Sequence[float]) -> float:
    return sum(100.0 * (x[i + 1] - x[i] ** 2) ** 2 + (1.0 - x[i]) ** 2 for i in range(len(x) - 1))


def rastrigin(x: Sequence[float]) -> float:
    n = len(x)
    return 10.0 * n + sum(v * v - 10.0 * math.cos(2.0 * math.pi * v) for v in x)


def ackley(x: Sequence[float]) -> float:
    n = len(x)
    if n == 0:
        return 0.0
    s1 = sum(v * v for v in x)
    s2 = sum(math.cos(2.0 * math.pi * v) for v in x)
    return -20.0 * math.exp(-0.2 * math.sqrt(s1 / n)) - math.exp(s2 / n) + 20.0 + math.e


def griewank(x: Sequence[float]) -> float:
    s = sum(v * v for v in x) / 4000.0
    p = 1.0
    for i, v in enumerate(x, start=1):
        p *= math.cos(v / math.sqrt(i))
    return s - p + 1.0


def levy(x: Sequence[float]) -> float:
    w = [1.0 + (v - 1.0) / 4.0 for v in x]
    term1 = math.sin(math.pi * w[0]) ** 2
    term3 = (w[-1] - 1.0) ** 2 * (1.0 + math.sin(2.0 * math.pi * w[-1]) ** 2)
    term2 = sum((wi - 1.0) ** 2 * (1.0 + 10.0 * math.sin(math.pi * wi + 1.0) ** 2) for wi in w[:-1])
    return term1 + term2 + term3


def default_problems(dim: int) -> list[BenchmarkProblem]:
    return [
        BenchmarkProblem("sphere", sphere, [-5.12] * dim, [5.12] * dim),
        BenchmarkProblem("rosenbrock", rosenbrock, [-2.0] * dim, [2.0] * dim),
        BenchmarkProblem("rastrigin", rastrigin, [-5.12] * dim, [5.12] * dim),
        BenchmarkProblem("ackley", ackley, [-32.768] * dim, [32.768] * dim),
        BenchmarkProblem("griewank", griewank, [-600.0] * dim, [600.0] * dim),
        BenchmarkProblem("levy", levy, [-10.0] * dim, [10.0] * dim),
    ]


def _opt_mig1(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if mig1 is None:
        try:
            import globalopt_native as _native
            return float(_native.mig1_py(list(a), list(b), int(budget), objective)["best_f"])
        except Exception as exc:
            raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_mig1") from exc
    del seed
    return float(mig1(a, b, budget, objective).best_f)


def _opt_mig2(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if mig2 is None:
        try:
            import globalopt_native as _native
            return float(_native.mig2_py(list(a), list(b), int(budget), objective)["best_f"])
        except Exception as exc:
            raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_mig2") from exc
    del seed
    return float(mig2(a, b, budget, objective).best_f)


def _opt_bayes1(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if bayes1 is None:
        try:
            import globalopt_native as _native
            init = max(2, min(20, budget // 5))
            return float(_native.bayes1_py(list(a), list(b), int(budget), int(init), objective, None)["best_f"])
        except Exception as exc:
            raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_bayes1") from exc
    del seed
    init = max(2, min(20, budget // 5))
    return float(bayes1(a, b, budget, init, objective).best_f)


def _opt_glopt(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if glopt is None:
        try:
            import globalopt_native as _native
            init = max(10, min(80, budget // 5))
            return float(_native.glopt_py(list(a), list(b), int(budget), int(init), 6, 0.92, objective)["best_f"])
        except Exception as exc:
            raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_glopt") from exc
    del seed
    init = max(10, min(80, budget // 5))
    return float(glopt(a, b, budget, init, 6, 0.92, objective).best_f)


def _midpoint(a: Sequence[float], b: Sequence[float]) -> list[float]:
    return [(ai + bi) * 0.5 for ai, bi in zip(a, b)]


def _opt_lpmin(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if lpmin is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_lpmin")
    del seed
    analysis = max(0, min(50, budget // 4))
    search = max(1, budget - analysis)
    return float(lpmin(a, b, analysis, search, objective).best_f)


def _opt_unt(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if unt is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_unt")
    del seed
    return float(unt(a, b, max(1, budget), 0.15, objective).best_f)


def _opt_exkor(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if exkor is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_exkor")
    del seed
    x0 = _midpoint(a, b)
    iterations = max(10, budget // 2)
    return float(exkor(x0, a, b, iterations, 0.25, 0.8, objective).best_f)


def _opt_extr(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if extr is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_extr")
    del seed
    x0 = _midpoint(a, b)
    return float(extr(x0, a, b, max(1, budget), objective).best_f)


def _opt_mivar4(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if mivar4 is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_mivar4")
    del seed
    x0 = _midpoint(a, b)
    iterations = max(10, budget // 2)
    return float(mivar4(x0, a, b, iterations, 0.1, objective).best_f)


def _opt_flexi(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if flexi is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_flexi")
    del seed
    x0 = _midpoint(a, b)
    iterations = max(20, budget)
    return float(flexi(x0, a, b, iterations, 0.08, objective).best_f)


def _opt_reqp(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if reqp is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_reqp")
    del seed
    x0 = _midpoint(a, b)
    iterations = max(10, budget // 2)
    return float(reqp(x0, a, b, iterations, 10.0, 1.25, objective, lambda _x: []).best_f)


def _opt_lbayes(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    if lbayes is None:
        raise RuntimeError("globalopt native module is unavailable; cannot run globalopt_lbayes")
    del seed
    init = max(10, min(80, budget // 5))
    local_it = max(20, min(120, budget // 2))
    return float(lbayes(a, b, max(1, budget), init, local_it, objective).best_f)


def random_search_optimizer(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    import random

    rng = random.Random(seed)
    best = float("inf")
    for _ in range(max(1, budget)):
        x = [ai + rng.random() * (bi - ai) for ai, bi in zip(a, b)]
        val = float(objective(x))
        if val < best:
            best = val
    return best


def scipy_de_optimizer(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    from scipy.optimize import differential_evolution

    bounds = list(zip(a, b))
    maxiter = max(1, budget // max(1, 15 * len(a)))
    popsize = max(5, min(20, budget // max(1, maxiter * len(a))))
    res = differential_evolution(objective, bounds=bounds, maxiter=maxiter, popsize=popsize, seed=seed, polish=False)
    return float(res.fun)


def nevergrad_optimizer(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    import nevergrad as ng

    param = ng.p.Array(shape=(len(a),)).set_bounds(list(a), list(b))
    # OnePlusOne is deterministic enough for small-budget smoke comparisons and avoids
    # metamodel internals that can fail on some NumPy/scikit-learn combinations.
    opt = ng.optimizers.OnePlusOne(parametrization=param, budget=max(1, budget), num_workers=1)
    opt.parametrization.random_state.seed(seed)

    def _obj(x: Any) -> float:
        if hasattr(x, "tolist"):
            x = x.tolist()
        if not isinstance(x, (list, tuple)):
            x = [x]
        return float(objective(x))

    recommendation = opt.minimize(_obj)
    best_x = recommendation.value.tolist() if hasattr(recommendation.value, "tolist") else list(recommendation.value)
    return float(objective(best_x))


def deap_optimizer(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    import random
    from deap import algorithms, base, creator, tools

    random.seed(seed)
    dim = len(a)
    pop_size = max(20, min(80, budget // 3 if budget > 0 else 20))
    ngen = max(1, budget // max(1, pop_size))

    if not hasattr(creator, "BenchmarkFitnessMin"):
        creator.create("BenchmarkFitnessMin", base.Fitness, weights=(-1.0,))
    if not hasattr(creator, "BenchmarkIndividual"):
        creator.create("BenchmarkIndividual", list, fitness=creator.BenchmarkFitnessMin)

    toolbox = base.Toolbox()

    def _attr(i: int) -> float:
        return random.uniform(a[i], b[i])

    toolbox.register("individual", tools.initIterate, creator.BenchmarkIndividual, lambda: [_attr(i) for i in range(dim)])
    toolbox.register("population", tools.initRepeat, list, toolbox.individual)
    toolbox.register("evaluate", lambda ind: (float(objective(ind)),))
    toolbox.register("mate", tools.cxBlend, alpha=0.5)
    toolbox.register("mutate", tools.mutPolynomialBounded, eta=20.0, low=list(a), up=list(b), indpb=1.0 / max(1, dim))
    toolbox.register("select", tools.selTournament, tournsize=3)

    pop = toolbox.population(n=pop_size)
    hof = tools.HallOfFame(1)
    algorithms.eaSimple(pop, toolbox, cxpb=0.7, mutpb=0.3, ngen=ngen, halloffame=hof, verbose=False)
    if len(hof) == 0:
        return float("inf")
    return float(objective(hof[0]))


def optuna_optimizer(a: Sequence[float], b: Sequence[float], budget: int, objective: Objective, seed: int) -> float:
    import optuna

    optuna.logging.set_verbosity(optuna.logging.WARNING)
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(direction="minimize", sampler=sampler)

    def _objective(trial: optuna.Trial) -> float:
        x = [trial.suggest_float(f"x{i}", a[i], b[i]) for i in range(len(a))]
        return float(objective(x))

    study.optimize(_objective, n_trials=max(1, budget), show_progress_bar=False)
    return float(study.best_value)


def default_optimizers() -> dict[str, Optimizer]:
    return {
        "globalopt_mig1": _opt_mig1,
        "globalopt_mig2": _opt_mig2,
        "globalopt_bayes1": _opt_bayes1,
        "globalopt_lpmin": _opt_lpmin,
        "globalopt_glopt": _opt_glopt,
        "globalopt_unt": _opt_unt,
        "globalopt_exkor": _opt_exkor,
        "globalopt_extr": _opt_extr,
        "globalopt_mivar4": _opt_mivar4,
        "globalopt_flexi": _opt_flexi,
        "globalopt_reqp": _opt_reqp,
        "globalopt_lbayes": _opt_lbayes,
    }


def run_benchmarks(
    dimensions: Sequence[int] = (2, 10, 30),
    budgets: Sequence[int] = (1000, 10000),
    seeds: Sequence[int] = tuple(range(1, 31)),
    optimizers: dict[str, Optimizer] | None = None,
    include_scipy_de: bool = False,
    include_nevergrad: bool = False,
    include_deap: bool = False,
    include_optuna: bool = False,
    success_tol: float = 1.0e-4,
) -> list[BenchmarkSummary]:
    opt_map = dict(default_optimizers() if optimizers is None else optimizers)
    if include_scipy_de:
        opt_map["scipy_differential_evolution"] = scipy_de_optimizer
    if include_nevergrad:
        opt_map["nevergrad_ngopt"] = nevergrad_optimizer
    if include_deap:
        opt_map["deap_ga"] = deap_optimizer
    if include_optuna:
        opt_map["optuna_tpe"] = optuna_optimizer

    summaries: list[BenchmarkSummary] = []

    for dim in dimensions:
        for problem in default_problems(dim):
            for budget in budgets:
                for opt_name, opt in opt_map.items():
                    best_vals: list[float] = []
                    times: list[float] = []
                    successes = 0
                    for seed in seeds:
                        t0 = time.perf_counter()
                        best = float(opt(problem.lower, problem.upper, int(budget), problem.objective, int(seed)))
                        dt = time.perf_counter() - t0
                        best_vals.append(best)
                        times.append(dt)
                        if best <= problem.optimum + success_tol:
                            successes += 1

                    summaries.append(
                        BenchmarkSummary(
                            optimizer=opt_name,
                            problem=problem.name,
                            dimension=int(dim),
                            budget=int(budget),
                            runs=len(seeds),
                            median_best=float(median(best_vals)),
                            best_of_runs=float(min(best_vals)),
                            success_rate=successes / max(1, len(seeds)),
                            median_seconds=float(median(times)),
                        )
                    )

    return summaries


def benchmark_table(summaries: Sequence[BenchmarkSummary]) -> list[dict[str, Any]]:
    return [
        {
            "optimizer": s.optimizer,
            "problem": s.problem,
            "dimension": s.dimension,
            "budget": s.budget,
            "runs": s.runs,
            "median_best": s.median_best,
            "best_of_runs": s.best_of_runs,
            "success_rate": s.success_rate,
            "median_seconds": s.median_seconds,
        }
        for s in summaries
    ]


def write_benchmark_csv(path: str | Path, summaries: Sequence[BenchmarkSummary]) -> Path:
    out = Path(path)
    out.parent.mkdir(parents=True, exist_ok=True)
    rows = benchmark_table(summaries)
    if not rows:
        out.write_text("", encoding="utf-8")
        return out

    fieldnames = list(rows[0].keys())
    with out.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    return out


def write_benchmark_markdown(path: str | Path, summaries: Sequence[BenchmarkSummary]) -> Path:
    out = Path(path)
    out.parent.mkdir(parents=True, exist_ok=True)
    rows = benchmark_table(summaries)
    if not rows:
        out.write_text("", encoding="utf-8")
        return out

    cols = list(rows[0].keys())
    lines = []
    lines.append("| " + " | ".join(cols) + " |")
    lines.append("| " + " | ".join(["---"] * len(cols)) + " |")
    for row in rows:
        lines.append("| " + " | ".join(str(row[c]) for c in cols) + " |")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out
