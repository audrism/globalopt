from pathlib import Path

from globalopt.benchmarks import (
    BenchmarkSummary,
    benchmark_table,
    random_search_optimizer,
    run_benchmarks,
    sphere,
    write_benchmark_csv,
    write_benchmark_markdown,
)


def test_objective_sphere_origin_is_zero() -> None:
    assert sphere([0.0, 0.0, 0.0]) == 0.0


def test_run_benchmarks_with_random_search_is_deterministic_shape() -> None:
    summaries = run_benchmarks(
        dimensions=(2,),
        budgets=(25,),
        seeds=(1, 2),
        optimizers={"random": random_search_optimizer},
        include_scipy_de=False,
    )

    # default_problems(2) yields 6 benchmark problems.
    assert len(summaries) == 6
    assert all(s.optimizer == "random" for s in summaries)
    assert all(s.dimension == 2 for s in summaries)
    assert all(s.budget == 25 for s in summaries)
    assert all(s.runs == 2 for s in summaries)


def test_write_benchmark_reports(tmp_path: Path) -> None:
    summaries = [
        BenchmarkSummary(
            optimizer="random",
            problem="sphere",
            dimension=2,
            budget=10,
            runs=2,
            median_best=0.5,
            best_of_runs=0.1,
            success_rate=0.0,
            median_seconds=0.001,
        )
    ]

    csv_path = tmp_path / "out" / "bench.csv"
    md_path = tmp_path / "out" / "bench.md"

    write_benchmark_csv(csv_path, summaries)
    write_benchmark_markdown(md_path, summaries)

    rows = benchmark_table(summaries)
    assert rows[0]["problem"] == "sphere"
    assert csv_path.exists()
    assert md_path.exists()
    assert "optimizer,problem" in csv_path.read_text(encoding="utf-8")
    assert "| optimizer | problem |" in md_path.read_text(encoding="utf-8")
