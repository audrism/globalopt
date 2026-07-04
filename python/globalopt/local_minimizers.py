from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Sequence, Protocol, runtime_checkable

Objective = Callable[[Sequence[float]], float]


@dataclass
class LocalMinimizeResult:
    par: list[float]
    value: float
    convergence: int = 0
    message: str = ""


@runtime_checkable
class LocalMinimizer(Protocol):
    def __call__(
        self,
        objective: Objective,
        x0: Sequence[float],
        lower: Sequence[float],
        upper: Sequence[float],
    ) -> LocalMinimizeResult:
        ...


def local_minimize(
    objective: Objective,
    x0: Sequence[float],
    lower: Sequence[float],
    upper: Sequence[float],
    local_minimizer: LocalMinimizer | None = None,
) -> LocalMinimizeResult:
    if local_minimizer is None:
        local_minimizer = scipy_local_minimizer()
    return local_minimizer(objective, x0, lower, upper)


def scipy_local_minimizer(method: str = "L-BFGS-B") -> LocalMinimizer:
    def _minimizer(
        objective: Objective,
        x0: Sequence[float],
        lower: Sequence[float],
        upper: Sequence[float],
    ) -> LocalMinimizeResult:
        try:
            from scipy.optimize import minimize
        except ImportError as exc:  # pragma: no cover - optional dependency
            raise RuntimeError("scipy is required for scipy_local_minimizer") from exc

        bounds = list(zip(lower, upper))
        result = minimize(lambda x: objective(list(x)), x0=list(x0), method=method, bounds=bounds)
        return LocalMinimizeResult(
            par=list(result.x),
            value=float(result.fun),
            convergence=int(getattr(result, "status", 0)),
            message=str(getattr(result, "message", "")),
        )

    return _minimizer
