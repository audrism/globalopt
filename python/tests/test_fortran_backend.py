"""Tests for backend="fortran" (the original GlobalMinimum routines).

Reference values from docs/FORTRAN_INTERFACES.md (builtin FURASN,
a=(-0.25,-0.125), b=(0.5,0.625)).
"""

import math

import pytest

import globalopt as go

pytestmark = pytest.mark.skipif(
    not go.HAS_FORTRAN, reason="native extension built without the Fortran backend"
)

A2 = [-0.25, -0.125]
B2 = [0.5, 0.625]


def test_lptau_reference_points():
    expected = [
        (0.5, 0.5),
        (0.25, 0.75),
        (0.75, 0.25),
        (0.125, 0.625),
    ]
    for i, exp in enumerate(expected, start=1):
        p = go.lp_tau_point(i, 2, backend="fortran")
        assert p == pytest.approx(exp, abs=1e-15)


def test_bayes1_builtin_reference():
    result = go.bayes1(A2, B2, 200, 20, objective="furasn", backend="fortran")
    assert result.best_f == pytest.approx(-1.9966149827762463, abs=1e-12)
    assert result.evals == 200
    assert len(result.values) == 200
    assert result.values[:3] == pytest.approx(
        [0.91709442215351888, -0.21484719829221782, -1.1207175656694048], abs=1e-12
    )


def test_mig2_builtin_reference():
    result = go.mig2(A2, B2, 200, objective="furasn", backend="fortran")
    assert result.best_f == pytest.approx(-1.8296951132230181, abs=1e-12)
    assert result.evals == 200


def test_python_callable_matches_builtin():
    calls = []

    def rastrigin(x):
        calls.append(1)
        n = len(x)
        return (2.0 / n) * sum(v * v - math.cos(18.0 * v) for v in x)

    builtin = go.bayes1(A2, B2, 200, 20, objective="furasn", backend="fortran")
    from_python = go.bayes1(A2, B2, 200, 20, objective=rastrigin, backend="fortran")
    assert len(calls) == 200
    assert from_python.best_f == builtin.best_f
    assert from_python.best_x == pytest.approx(builtin.best_x, abs=0.0)


def test_ats_state_seeding_mig2():
    seed = [0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77, 0.88, 0.99, 0.12, 0.23, 0.34, 0.45, 0.56, 0.67]
    default = go.mig2(A2, B2, 200, objective="furasn", backend="fortran")
    seeded1 = go.mig2(A2, B2, 200, objective="furasn", backend="fortran", ats_state=seed)
    seeded2 = go.mig2(A2, B2, 200, objective="furasn", backend="fortran", ats_state=seed)
    # Different seed changes the result; the same seed reproduces it.
    assert seeded1.best_f != default.best_f
    assert seeded1.best_f == seeded2.best_f
    assert seeded1.best_x == seeded2.best_x


def test_extr_is_one_dimensional():
    result = go.extr(-0.25, 0.5, 100, objective="furasn", backend="fortran")
    assert len(result.best_x) == 1
    assert result.best_f < -1.8  # FURASN 1-D global minimum is -2 at x=0


def test_unt_glopt_lpmin_fortran():
    u = go.unt(A2, B2, 120, backend="fortran", objective="furasn")
    assert u.best_f < -1.0

    g = go.glopt(A2, B2, 300, objective="furasn", backend="fortran", start_points=10)
    assert g.best_f < -1.5

    lp = go.lpmin(A2, B2, 50, 100, objective="furasn", backend="fortran")
    assert lp.best_f < -1.5


def test_exkor_mivar4_lbayes_fortran():
    def sphere(x):
        return sum(v * v for v in x)

    ek = go.exkor([0.3, 0.4], [-1.0, -1.0], [1.0, 1.0], objective="furasn", backend="fortran")
    assert ek.best_f < -1.5

    mv = go.mivar4([0.3, 0.4], [-1.0, -1.0], [1.0, 1.0], objective=sphere, backend="fortran")
    assert mv.best_f < 1e-6

    lb = go.lbayes([-1.0, -1.0], [1.0, 1.0], objective=sphere, backend="fortran", x0=[0.3, 0.4])
    assert len(lb.best_x) == 2
    assert math.isfinite(lb.best_f)


def test_flexi_and_reqp_constrained_fortran():
    def sphere(x):
        return sum(v * v for v in x)

    def constraints(x):
        # One inequality: x + y - 1 >= 0.
        return [x[0] + x[1] - 1.0]

    fx = go.flexi(
        [0.3, 0.4],
        objective=sphere,
        backend="fortran",
        max_evals=200,
        n_eq=0,
        n_ineq=1,
        size=0.3,
        conver=1e-6,
        constraints=constraints,
    )
    assert fx.best_x[0] + fx.best_x[1] > 0.99
    assert fx.best_f == pytest.approx(0.5, abs=0.05)

    rq = go.reqp(
        [0.3, 0.4],
        objective=sphere,
        constr=constraints,
        backend="fortran",
        imax=50,
        n_eq=0,
        n_ineq=1,
    )
    assert rq.best_f == pytest.approx(0.5, abs=1e-3)


def test_numpy_arrays_accepted():
    np = pytest.importorskip("numpy")
    result = go.mig2(np.asarray(A2), np.asarray(B2), 200, objective="furasn", backend="fortran")
    assert result.best_f == pytest.approx(-1.8296951132230181, abs=1e-12)


def test_invalid_inputs_raise():
    with pytest.raises(ValueError):
        go.mig2([0.0] * 21, [1.0] * 21, 100, objective="furasn", backend="fortran")
    with pytest.raises(ValueError):
        go.bayes1(A2, B2, 1001, 20, objective="furasn", backend="fortran")
    with pytest.raises(ValueError):
        go.mig2(A2, B2, 200, objective="nosuch", backend="fortran")
    with pytest.raises(ValueError):
        go.mig2(A2, B2, 200, objective="furasn", backend="fortran", ats_state=[0.5] * 14)
    with pytest.raises(ValueError):
        go.mig2(A2, B2, 200, objective="furasn", backend="nope")
