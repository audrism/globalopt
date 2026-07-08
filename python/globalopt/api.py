"""High-level Python API over the globalopt native extension.

Every optimizer accepts a ``backend`` argument:

* ``backend="rust"`` (default for pre-0.2 signatures) runs the Rust
  translation of the GlobalMinimum routines.  For several methods the
  Rust version is a modernized variant, not a line-for-line port.
* ``backend="fortran"`` runs the original 1989 Fortran routines (Mockus,
  GlobalMinimum) compiled into the extension.  This is the genuine
  algorithm for every method.  See the backends table in README.md.

With ``backend="fortran"`` the ``objective`` may be either a Python
callable or the name of a compiled builtin test objective (``"furasn"``,
``"fush5"``, ``"fush7"``, ``"fush10"``, ``"fuhar3"``, ``"fuhar6"``,
``"fubran"``, ``"fugold"``); builtins avoid any Python-callback overhead.
``ats_state`` (15 floats) seeds the ATS random generator for
reproducible runs; by default each call starts from the canonical
fresh-process state.

Bounds, start points and ``ats_state`` accept any sequence of numbers,
including numpy arrays (no numpy dependency required).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Iterable, Sequence, Union

import globalopt_native as _native

Objective = Union[Callable[[Sequence[float]], float], str]
Constraints = Callable[[Sequence[float]], Sequence[float]]

#: Names of compiled builtin objectives usable as ``objective=`` strings
#: with backend="fortran".
BUILTIN_OBJECTIVES = (
    "furasn",
    "fush5",
    "fush7",
    "fush10",
    "fuhar3",
    "fuhar6",
    "fubran",
    "fugold",
)

#: True when the native extension was built with the original Fortran
#: library (feature "fortran").
HAS_FORTRAN: bool = bool(getattr(_native, "HAS_FORTRAN", False))


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
fush5 = _native.fush5_py
fush7 = _native.fush7_py
fush10 = _native.fush10_py
fuhar3 = _native.fuhar3_py
fuhar6 = _native.fuhar6_py
fubran = _native.fubran_py
fugold = _native.fugold_py

_RUST_BUILTINS: dict[str, Callable[[Sequence[float]], float]] = {
    "furasn": furasn,
    "fush5": fush5,
    "fush7": fush7,
    "fush10": fush10,
    "fuhar3": fuhar3,
    "fuhar6": fuhar6,
    "fubran": fubran,
    "fugold": fugold,
}


def _floats(x: Iterable[float], name: str) -> list[float]:
    try:
        return [float(v) for v in x]
    except TypeError as exc:  # not iterable / non-numeric entries
        raise TypeError(f"{name} must be a sequence of numbers") from exc


def _opt_floats(x: Iterable[float] | None, name: str) -> list[float] | None:
    return None if x is None else _floats(x, name)


def _check_backend(backend: str) -> str:
    if backend not in ("rust", "fortran"):
        raise ValueError(f"backend must be 'rust' or 'fortran', got {backend!r}")
    if backend == "fortran" and not HAS_FORTRAN:
        raise RuntimeError(
            "the native extension was built without the Fortran backend"
        )
    return backend


def _rust_objective(objective: Objective) -> Callable[[Sequence[float]], float]:
    """The rust backend needs a callable; map builtin names to the
    corresponding native test functions."""
    if isinstance(objective, str):
        try:
            return _RUST_BUILTINS[objective]
        except KeyError:
            raise ValueError(
                f"unknown builtin objective {objective!r}; expected one of "
                f"{', '.join(BUILTIN_OBJECTIVES)}"
            ) from None
    return objective


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


def lp_tau_point(c: int, n: int, backend: str = "rust") -> list[float]:
    """Point ``c`` (1-based) of the n-dimensional LP-tau sequence."""
    if _check_backend(backend) == "fortran":
        return list(_native.lptau_fortran_py(c, n))
    return list(_native.lp_tau_point_py(c, n))


def mig1(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    objective: Objective = furasn,
    backend: str = "rust",
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Pure Monte-Carlo search (MIG1)."""
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.mig1_fortran_py(a, b, evaluations, objective, _opt_floats(ats_state, "ats_state"))
        )
    return _wrap_opt_result(_native.mig1_py(a, b, evaluations, _rust_objective(objective)))


def mig2(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    objective: Objective = furasn,
    backend: str = "rust",
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Monte-Carlo search keeping the second minimum (MIG2)."""
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.mig2_fortran_py(a, b, evaluations, objective, _opt_floats(ats_state, "ats_state"))
        )
    return _wrap_opt_result(_native.mig2_py(a, b, evaluations, _rust_objective(objective)))


def bayes1(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    initial_points: int,
    objective: Objective = furasn,
    local_minimizer: Callable[..., Any] | None = None,
    backend: str = "rust",
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """One-step Bayesian global search (BAYES1)."""
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        if local_minimizer is not None:
            raise ValueError("local_minimizer is only supported with backend='rust'")
        return _wrap_opt_result(
            _native.bayes1_fortran_py(
                a, b, evaluations, initial_points, objective, _opt_floats(ats_state, "ats_state")
            )
        )
    return _wrap_opt_result(
        _native.bayes1_py(a, b, evaluations, initial_points, _rust_objective(objective), local_minimizer)
    )


def lpmin(
    a: Sequence[float],
    b: Sequence[float],
    analysis_evals: int,
    search_evals: int,
    objective: Objective = furasn,
    backend: str = "rust",
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Deterministic LP-tau search (LPMIN).

    With backend="fortran", ``analysis_evals`` follows the original
    convention: negative = no factor analysis, 0 = natural variable
    order, 10..300 = number of analysis evaluations.
    """
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.lpmin_fortran_py(
                a, b, analysis_evals, search_evals, objective, _opt_floats(ats_state, "ats_state")
            )
        )
    return _wrap_opt_result(
        _native.lpmin_py(a, b, max(analysis_evals, 0), search_evals, _rust_objective(objective))
    )


def glopt(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    initial_points: int = 80,
    local_trials: int = 5,
    shrink: float = 0.92,
    objective: Objective = furasn,
    backend: str = "rust",
    start_points: int = 10,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Clustering global method (GLOPT).

    backend="rust" uses ``initial_points``/``local_trials``/``shrink``;
    backend="fortran" uses ``evaluations`` and ``start_points`` (PN,
    1..150) of the original routine.
    """
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.glopt_fortran_py(
                a, b, evaluations, start_points, objective, _opt_floats(ats_state, "ats_state")
            )
        )
    return _wrap_opt_result(
        _native.glopt_py(a, b, evaluations, initial_points, local_trials, shrink, _rust_objective(objective))
    )


def unt(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int,
    local_step: float = 0.15,
    objective: Objective = furasn,
    backend: str = "rust",
    initial_points: int = 0,
    max_local_minima: int = 5,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Uniform deterministic grid search (UNT).

    backend="rust" uses ``local_step``; backend="fortran" uses
    ``initial_points`` (0 = automatic, else 30..evaluations) and
    ``max_local_minima`` (1..20) of the original routine.
    """
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.unt_fortran_py(
                a,
                b,
                evaluations,
                initial_points,
                max_local_minima,
                objective,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    return _wrap_opt_result(_native.unt_py(a, b, evaluations, local_step, _rust_objective(objective)))


def exkor(
    x0: Sequence[float],
    a: Sequence[float],
    b: Sequence[float],
    iterations: int = 120,
    step: float = 0.25,
    shrink: float = 0.8,
    objective: Objective = furasn,
    backend: str = "rust",
    evals_per_coord: int = 100,
    model_evals: int = 6,
    cycles: int = 2,
    first_coord: int = 1,
    acc: float = 0.01,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Coordinate optimization (EXKOR).

    backend="rust" is a +/- step coordinate descent driven by
    ``iterations``/``step``/``shrink``.  backend="fortran" runs the
    original method (each coordinate solved by an EXTR-style 1-D Wiener
    search) driven by ``evals_per_coord``, ``model_evals`` (>=6),
    ``cycles``, ``first_coord`` (1-based) and accuracy ``acc``.
    """
    x0, a, b = _floats(x0, "x0"), _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.exkor_fortran_py(
                x0,
                a,
                b,
                evals_per_coord,
                model_evals,
                cycles,
                first_coord,
                acc,
                objective,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    return _wrap_opt_result(
        _native.exkor_py(x0, a, b, iterations, step, shrink, _rust_objective(objective))
    )


def extr(
    bp: float,
    ep: float,
    evaluations: int,
    model_evals: int = 6,
    acc_y: float = 0.01,
    acc_x: float = 0.01,
    objective: Objective = furasn,
    backend: str = "fortran",
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """1-D global search on the interval [bp, ep] (EXTR).

    .. note:: signature changed in 0.2.0: EXTR is one-dimensional, as in
       the original library.  backend="fortran" (default) runs the
       original Wiener-process model search; backend="rust" runs the
       modernized 1-D variant on the same interval.
    """
    bp, ep = float(bp), float(ep)
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.extr_fortran_py(
                bp,
                ep,
                evaluations,
                model_evals,
                acc_y,
                acc_x,
                objective,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    return _wrap_opt_result(
        _native.extr_py([(bp + ep) / 2.0], [bp], [ep], evaluations, _rust_objective(objective))
    )


def mivar4(
    x0: Sequence[float],
    a: Sequence[float],
    b: Sequence[float],
    iterations: int = 120,
    step: float = 0.1,
    objective: Objective = furasn,
    backend: str = "rust",
    max_evals: int = 100,
    nstop: int = 2,
    imax: int = 100,
    xeps: float = 100.0,
    eps: float = 1e-4,
    eps1: float = 1e-4,
    delta: float = 1e-4,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Variable-metric local method (MIVAR4).

    backend="rust" uses ``iterations``/``step``; backend="fortran" runs
    the original method with ``max_evals``, ``nstop``, ``imax`` and the
    tolerances ``xeps``/``eps``/``eps1``/``delta``.
    """
    x0, a, b = _floats(x0, "x0"), _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.mivar4_fortran_py(
                x0,
                a,
                b,
                max_evals,
                nstop,
                imax,
                xeps,
                eps,
                eps1,
                delta,
                objective,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    return _wrap_opt_result(_native.mivar4_py(x0, a, b, iterations, step, _rust_objective(objective)))


def flexi(
    x0: Sequence[float],
    a: Sequence[float] | None = None,
    b: Sequence[float] | None = None,
    iterations: int = 180,
    simplex_scale: float = 0.08,
    objective: Objective = furasn,
    backend: str = "rust",
    max_evals: int = 50,
    n_eq: int = 0,
    n_ineq: int = 0,
    size: float = 0.3,
    conver: float = 1e-5,
    constraints: Constraints | None = None,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Flexible-tolerance simplex (FLEXI).

    backend="rust" is a bounded Nelder-Mead using ``a``/``b``/
    ``iterations``/``simplex_scale``.  backend="fortran" runs the
    original constrained flexible-tolerance method: NO bounds (the
    region is set by ``x0`` and ``size``); ``constraints(x)`` must
    return ``n_eq`` equality values (=0 feasible) followed by ``n_ineq``
    inequality values (>=0 feasible).
    """
    x0 = _floats(x0, "x0")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.flexi_fortran_py(
                x0,
                max_evals,
                n_eq,
                n_ineq,
                size,
                conver,
                objective,
                constraints,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    if a is None or b is None:
        raise ValueError("backend='rust' requires bounds a and b")
    return _wrap_opt_result(
        _native.flexi_py(x0, _floats(a, "a"), _floats(b, "b"), iterations, simplex_scale, _rust_objective(objective))
    )


def reqp(
    x0: Sequence[float],
    a: Sequence[float] | None = None,
    b: Sequence[float] | None = None,
    iterations: int = 120,
    penalty: float = 10.0,
    penalty_growth: float = 1.25,
    objective: Objective = furasn,
    constr: Constraints | None = None,
    backend: str = "rust",
    imax: int = 50,
    n_eq: int = 0,
    n_ineq: int = 0,
    r1: float = 1.0,
    scale: float = 0.25,
    delta: float = 1e-4,
    eps: float = 1e-4,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Constrained minimization (REQP).

    backend="rust" is a quadratic-penalty variant with bounds.
    backend="fortran" runs the original recursive quadratic programming
    method: NO bounds; ``constr(x)`` returns ``n_eq`` equality values
    (=0 feasible) followed by ``n_ineq`` inequality values (>=0
    feasible); controlled by ``imax``, ``r1``, ``scale``, ``delta``,
    ``eps``.
    """
    x0 = _floats(x0, "x0")
    if _check_backend(backend) == "fortran":
        return _wrap_opt_result(
            _native.reqp_fortran_py(
                x0,
                imax,
                n_eq,
                n_ineq,
                r1,
                scale,
                delta,
                eps,
                objective,
                constr,
                _opt_floats(ats_state, "ats_state"),
            )
        )
    if a is None or b is None:
        raise ValueError("backend='rust' requires bounds a and b")
    if constr is None:
        raise ValueError("backend='rust' requires a constr callable")
    return _wrap_opt_result(
        _native.reqp_py(
            x0, _floats(a, "a"), _floats(b, "b"), iterations, penalty, penalty_growth, _rust_objective(objective), constr
        )
    )


def lbayes(
    a: Sequence[float],
    b: Sequence[float],
    evaluations: int = 220,
    initial_points: int = 30,
    local_iterations: int = 80,
    objective: Objective = furasn,
    local_minimizer: Callable[..., Any] | None = None,
    backend: str = "rust",
    x0: Sequence[float] | None = None,
    iterations: int = 5,
    aniu: float = 0.05,
    beta: float = 0.9,
    ats_state: Sequence[float] | None = None,
) -> OptResult:
    """Bayesian method with local descent (LBAYES).

    backend="rust" uses ``evaluations``/``initial_points``/
    ``local_iterations``.  backend="fortran" runs the original
    deterministic method from start point ``x0`` (defaults to the box
    midpoint) with ``iterations`` (IT), smoothing ``aniu`` and ``beta``;
    its best_f is the smoothed mean value the method minimizes.
    """
    a, b = _floats(a, "a"), _floats(b, "b")
    if _check_backend(backend) == "fortran":
        if local_minimizer is not None:
            raise ValueError("local_minimizer is only supported with backend='rust'")
        start = _opt_floats(x0, "x0")
        if start is None:
            start = [(ai + bi) / 2.0 for ai, bi in zip(a, b)]
        return _wrap_opt_result(
            _native.lbayes_fortran_py(
                start, a, b, iterations, aniu, beta, objective, _opt_floats(ats_state, "ats_state")
            )
        )
    return _wrap_opt_result(
        _native.lbayes_py(
            a, b, evaluations, initial_points, local_iterations, _rust_objective(objective), local_minimizer
        )
    )


def anal1(a: Sequence[float], b: Sequence[float], samples: int, objective: Objective = furasn) -> AnalResult:
    """Variable-influence screening (rust backend only)."""
    return _wrap_anal_result(
        _native.anal1_py(_floats(a, "a"), _floats(b, "b"), samples, _rust_objective(objective))
    )


def anal2(a: Sequence[float], b: Sequence[float], samples: int, objective: Objective = furasn) -> AnalResult:
    """Variance-based variable screening (rust backend only)."""
    return _wrap_anal_result(
        _native.anal2_py(_floats(a, "a"), _floats(b, "b"), samples, _rust_objective(objective))
    )
