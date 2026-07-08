"""Benchmark problem set, shared between the Python and R harnesses.

Each problem is defined on a box with known global optimum value 0 after
shifting: we minimize g(x) = f(x - delta) - f_opt where delta is a small
seeded shift (COCO-style instances), so the optimal value of every
instance is exactly 0 and gap-to-optimum is directly comparable.

The shift for (problem, instance) is generated with a Park-Miller LCG so
the R harness (problems.R) produces bit-identical instances.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Callable, Sequence

Vector = Sequence[float]


def _lcg_stream(seed: int, count: int) -> list[float]:
    """Park-Miller minimal standard generator, matching problems.R."""
    x = seed % 2147483647
    if x <= 0:
        x += 2147483646
    out = []
    for _ in range(count):
        x = (x * 16807) % 2147483647
        out.append(x / 2147483647)
    return out


# ---------------- scalable objectives (optimum 0 at 0 unless noted) ---


def sphere(x: Vector) -> float:
    return sum(v * v for v in x)


def rosenbrock(x: Vector) -> float:
    # optimum 0 at (1, ..., 1)
    return sum(
        100.0 * (x[i + 1] - x[i] ** 2) ** 2 + (1.0 - x[i]) ** 2
        for i in range(len(x) - 1)
    )


def rastrigin(x: Vector) -> float:
    return 10.0 * len(x) + sum(v * v - 10.0 * math.cos(2 * math.pi * v) for v in x)


def ackley(x: Vector) -> float:
    n = len(x)
    s1 = sum(v * v for v in x)
    s2 = sum(math.cos(2 * math.pi * v) for v in x)
    return (
        -20.0 * math.exp(-0.2 * math.sqrt(s1 / n))
        - math.exp(s2 / n)
        + 20.0
        + math.e
    )


def griewank(x: Vector) -> float:
    s = sum(v * v for v in x) / 4000.0
    p = 1.0
    for i, v in enumerate(x, start=1):
        p *= math.cos(v / math.sqrt(i))
    return s - p + 1.0


def levy(x: Vector) -> float:
    w = [1.0 + (v - 1.0) / 4.0 for v in x]
    t1 = math.sin(math.pi * w[0]) ** 2
    t3 = (w[-1] - 1.0) ** 2 * (1.0 + math.sin(2 * math.pi * w[-1]) ** 2)
    t2 = sum(
        (wi - 1.0) ** 2 * (1.0 + 10.0 * math.sin(math.pi * wi + 1.0) ** 2)
        for wi in w[:-1]
    )
    return t1 + t2 + t3


def schwefel(x: Vector) -> float:
    # optimum 0 at 420.9687...; use the standard form
    n = len(x)
    return 418.9829101183649 * n - sum(v * math.sin(math.sqrt(abs(v))) for v in x)


def zakharov(x: Vector) -> float:
    s1 = sum(v * v for v in x)
    s2 = sum(0.5 * (i + 1) * v for i, v in enumerate(x))
    return s1 + s2**2 + s2**4


# ---------------- fixed-dimension classics ---------------------------


def branin(x: Vector) -> float:
    # optimum 0.397887357729739 (subtracted by caller table)
    x1, x2 = x[0], x[1]
    a, b, c = 1.0, 5.1 / (4 * math.pi**2), 5.0 / math.pi
    r, s, t = 6.0, 10.0, 1.0 / (8 * math.pi)
    return a * (x2 - b * x1 * x1 + c * x1 - r) ** 2 + s * (1 - t) * math.cos(x1) + s


def goldstein_price(x: Vector) -> float:
    # optimum 3 at (0, -1)
    x1, x2 = x[0], x[1]
    t1 = 1 + (x1 + x2 + 1) ** 2 * (
        19 - 14 * x1 + 3 * x1 * x1 - 14 * x2 + 6 * x1 * x2 + 3 * x2 * x2
    )
    t2 = 30 + (2 * x1 - 3 * x2) ** 2 * (
        18 - 32 * x1 + 12 * x1 * x1 + 48 * x2 - 36 * x1 * x2 + 27 * x2 * x2
    )
    return t1 * t2


def six_hump_camel(x: Vector) -> float:
    # optimum -1.031628453489877
    x1, x2 = x[0], x[1]
    return (
        (4 - 2.1 * x1 * x1 + x1**4 / 3) * x1 * x1
        + x1 * x2
        + (-4 + 4 * x2 * x2) * x2 * x2
    )


_SHEKEL_A = [
    [4.0, 4.0, 4.0, 4.0],
    [1.0, 1.0, 1.0, 1.0],
    [8.0, 8.0, 8.0, 8.0],
    [6.0, 6.0, 6.0, 6.0],
    [3.0, 7.0, 3.0, 7.0],
    [2.0, 9.0, 2.0, 9.0],
    [5.0, 5.0, 3.0, 3.0],
    [8.0, 1.0, 8.0, 1.0],
    [6.0, 2.0, 6.0, 2.0],
    [7.0, 3.6, 7.0, 3.6],
]
_SHEKEL_C = [0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3, 0.7, 0.5, 0.5]


def _shekel(x: Vector, m: int) -> float:
    f = 0.0
    for i in range(m):
        d = _SHEKEL_C[i]
        for j in range(4):
            d += (x[j] - _SHEKEL_A[i][j]) ** 2
        f -= 1.0 / d
    return f


def shekel5(x: Vector) -> float:
    return _shekel(x, 5)  # optimum -10.153199679058231


def shekel7(x: Vector) -> float:
    return _shekel(x, 7)  # optimum -10.402940566818664


def shekel10(x: Vector) -> float:
    return _shekel(x, 10)  # optimum -10.536409816692023


_H3_ALPHA = [[3.0, 10.0, 30.0], [0.1, 10.0, 35.0], [3.0, 10.0, 30.0], [0.1, 10.0, 35.0]]
_H3_P = [
    [0.3689, 0.1170, 0.2673],
    [0.4699, 0.4387, 0.7470],
    [0.1091, 0.8732, 0.5547],
    [0.03815, 0.5743, 0.8828],
]
_H_C = [1.0, 1.2, 3.0, 3.2]
_H6_ALPHA = [
    [10.0, 3.0, 17.0, 3.5, 1.7, 8.0],
    [0.05, 10.0, 17.0, 0.1, 8.0, 14.0],
    [3.0, 3.5, 1.7, 10.0, 17.0, 8.0],
    [17.0, 8.0, 0.05, 10.0, 0.1, 14.0],
]
_H6_P = [
    [0.1312, 0.1696, 0.5569, 0.0124, 0.8283, 0.5886],
    [0.2329, 0.4135, 0.8307, 0.3736, 0.1004, 0.9991],
    [0.2348, 0.1451, 0.3522, 0.2883, 0.3047, 0.6650],
    [0.4047, 0.8828, 0.8732, 0.5743, 0.1091, 0.0381],
]


def _hartmann(x: Vector, alpha, p) -> float:
    f = 0.0
    for i in range(4):
        s = 0.0
        for j in range(len(x)):
            s -= alpha[i][j] * (x[j] - p[i][j]) ** 2
        f -= _H_C[i] * math.exp(s)
    return f


def hartmann3(x: Vector) -> float:
    return _hartmann(x, _H3_ALPHA, _H3_P)  # optimum -3.862782147820756


def hartmann6(x: Vector) -> float:
    return _hartmann(x, _H6_ALPHA, _H6_P)  # optimum -3.322368011391339


@dataclass(frozen=True)
class Problem:
    """One benchmark instance: shifted objective with optimum exactly 0."""

    name: str
    dim: int
    instance: int
    lower: tuple[float, ...]
    upper: tuple[float, ...]
    objective: Callable[[Vector], float]

    def __call__(self, x: Vector) -> float:
        return self.objective(x)


_SCALABLE: dict[str, tuple[Callable, float, float, float]] = {
    # name: (fn, lower, upper, f_opt); optimum location must be interior
    "sphere": (sphere, -5.12, 5.12, 0.0),
    "rosenbrock": (rosenbrock, -2.048, 2.048, 0.0),
    "rastrigin": (rastrigin, -5.12, 5.12, 0.0),
    "ackley": (ackley, -32.768, 32.768, 0.0),
    "griewank": (griewank, -600.0, 600.0, 0.0),
    "levy": (levy, -10.0, 10.0, 0.0),
    "schwefel": (schwefel, -500.0, 500.0, 0.0),
    "zakharov": (zakharov, -5.0, 10.0, 0.0),
}

_FIXED: dict[str, tuple[Callable, list[float], list[float], float]] = {
    "branin": (branin, [-5.0, 0.0], [10.0, 15.0], 0.39788735772973816),
    "goldstein_price": (goldstein_price, [-2.0, -2.0], [2.0, 2.0], 3.0),
    "six_hump_camel": (six_hump_camel, [-3.0, -2.0], [3.0, 2.0], -1.0316284534898774),
    "shekel5": (shekel5, [0.0] * 4, [10.0] * 4, -10.153199679058231),
    "shekel7": (shekel7, [0.0] * 4, [10.0] * 4, -10.402940566818664),
    "shekel10": (shekel10, [0.0] * 4, [10.0] * 4, -10.536409816692023),
    "hartmann3": (hartmann3, [0.0] * 3, [1.0] * 3, -3.8627821478207558),
    "hartmann6": (hartmann6, [0.0] * 6, [1.0] * 6, -3.3223680113913385),
}

SCALABLE_DIMS = (2, 5, 10)
SHIFT_FRACTION = 0.05  # shift by up to +/- 5% of the box per coordinate

# problem ids (stable, used in the shift LCG seed) — shared with problems.R
PROBLEM_IDS = {
    name: i + 1
    for i, name in enumerate(
        list(_SCALABLE.keys()) + list(_FIXED.keys())
    )
}


def make_problem(name: str, dim: int, instance: int) -> Problem:
    """Instance `instance` (>=0) of a problem; instance 0 is unshifted."""
    if name in _SCALABLE:
        fn, lo, hi, fopt = _SCALABLE[name]
        lower = [lo] * dim
        upper = [hi] * dim
    else:
        fn, lower, upper, fopt = _FIXED[name]
        lower = list(lower)
        upper = list(upper)
        if dim != len(lower):
            raise ValueError(f"{name} is fixed at dimension {len(lower)}")

    n = len(lower)
    if instance > 0:
        seed = PROBLEM_IDS[name] * 10000 + n * 100 + instance
        u = _lcg_stream(seed, n)
        delta = [
            (2.0 * u[j] - 1.0) * SHIFT_FRACTION * (upper[j] - lower[j])
            for j in range(n)
        ]
    else:
        delta = [0.0] * n

    def shifted(x: Vector, _fn=fn, _d=delta, _fopt=fopt) -> float:
        return _fn([x[j] - _d[j] for j in range(len(x))]) - _fopt

    return Problem(
        name=name,
        dim=n,
        instance=instance,
        lower=tuple(lower),
        upper=tuple(upper),
        objective=shifted,
    )


def default_problem_set(instances: int = 15) -> list[Problem]:
    """The full benchmark set: scalable problems at dims 2/5/10 plus the
    fixed-dimension classics, `instances` seeded instances each."""
    problems = []
    for name in _SCALABLE:
        for dim in SCALABLE_DIMS:
            for inst in range(1, instances + 1):
                problems.append(make_problem(name, dim, inst))
    for name, (fn, lo, hi, fopt) in _FIXED.items():
        for inst in range(1, instances + 1):
            problems.append(make_problem(name, len(lo), inst))
    return problems
