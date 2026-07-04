from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Sequence

import globalopt_native as _native

Objective = Callable[[Sequence[float]], float]


@dataclass
class OptResult:
    best_x: list[float]
    best_f: float
    evals: int
    best_iter: int
    points: list[list[float]]
    values: list[float]


@dataclass
class AnalResult:
    variable_order: list[int]
    influence_scores: list[float]
    samples: int


furasn = _native.furasn_py
lp_tau_point = _native.lp_tau_point_py


def _wrap_opt_result(result: dict[str, Any]) -> OptResult:
    return OptResult(
        best_x=list(result["best_x"]),
        best_f=float(result["best_f"]),
        evals=int(result["evals"]),
        best_iter=int(result["best_iter"]),
        points=[list(p) for p in result["points"]],
        values=[float(v) for v in result["values"]],
    )


def _wrap_anal_result(result: dict[str, Any]) -> AnalResult:
    return AnalResult(
        variable_order=[int(v) for v in result["variable_order"]],
        influence_scores=[float(v) for v in result["influence_scores"]],
        samples=int(result["samples"]),
    )


def mig1(a: Sequence[float], b: Sequence[float], evaluations: int, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.mig1_py(list(a), list(b), evaluations, objective))


def mig2(a: Sequence[float], b: Sequence[float], evaluations: int, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.mig2_py(list(a), list(b), evaluations, objective))


def bayes1(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    initial_points: int,
    objective: Objective = furasn,
    local_minimizer: Callable[..., Any] | None = None,
) -> OptResult:
    return _wrap_opt_result(
        _native.bayes1_py(list(a), list(b), evaluations, initial_points, objective, local_minimizer)
    )


def lpmin(a: Sequence[float], b: Sequence[float], analysis_evals: int, search_evals: int, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.lpmin_py(list(a), list(b), analysis_evals, search_evals, objective))


def glopt(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    initial_points: int,
    local_trials: int,
    shrink: float,
    objective: Objective = furasn,
) -> OptResult:
    return _wrap_opt_result(_native.glopt_py(list(a), list(b), evaluations, initial_points, local_trials, shrink, objective))


def unt(a: Sequence[float], b: Sequence[float], evaluations: int, local_step: float, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.unt_py(list(a), list(b), evaluations, local_step, objective))


def exkor(x0: Sequence[float], a: Sequence[float], b: Sequence[float], iterations: int, step: float, shrink: float, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.exkor_py(list(x0), list(a), list(b), iterations, step, shrink, objective))


def extr(x0: Sequence[float], a: Sequence[float], b: Sequence[float], evaluations: int, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.extr_py(list(x0), list(a), list(b), evaluations, objective))


def mivar4(x0: Sequence[float], a: Sequence[float], b: Sequence[float], iterations: int, step: float, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.mivar4_py(list(x0), list(a), list(b), iterations, step, objective))


def flexi(x0: Sequence[float], a: Sequence[float], b: Sequence[float], iterations: int, simplex_scale: float, objective: Objective = furasn) -> OptResult:
    return _wrap_opt_result(_native.flexi_py(list(x0), list(a), list(b), iterations, simplex_scale, objective))


def reqp(x0: Sequence[float], a: Sequence[float], b: Sequence[float], iterations: int, penalty: float, penalty_growth: float, objective: Objective, constr: Callable[[Sequence[float]], Sequence[float]]) -> OptResult:
    return _wrap_opt_result(_native.reqp_py(list(x0), list(a), list(b), iterations, penalty, penalty_growth, objective, constr))


def lbayes(a: Sequence[float], b: Sequence[float], evaluations: int, initial_points: int, local_iterations: int, objective: Objective = furasn, local_minimizer: Callable[..., Any] | None = None) -> OptResult:
    return _wrap_opt_result(_native.lbayes_py(list(a), list(b), evaluations, initial_points, local_iterations, objective, local_minimizer))


def anal1(a: Sequence[float], b: Sequence[float], samples: int, objective: Objective = furasn) -> AnalResult:
    return _wrap_anal_result(_native.anal1_py(list(a), list(b), samples, objective))


def anal2(a: Sequence[float], b: Sequence[float], samples: int, objective: Objective = furasn) -> AnalResult:
    return _wrap_anal_result(_native.anal2_py(list(a), list(b), samples, objective))
