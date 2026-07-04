from pathlib import Path

from globalopt.benchmarks import (
    benchmark_table,
    random_search_optimizer,
    run_benchmarks,
    write_benchmark_csv,
    write_benchmark_markdown,
)


def main() -> None:
    try:
        summaries = run_benchmarks(
            dimensions=(2, 10),
            budgets=(1000,),
            seeds=tuple(range(1, 6)),
            optimizers={"random_search": random_search_optimizer},
            include_scipy_de=True,
        )
    except Exception:
        summaries = run_benchmarks(
            dimensions=(2, 10),
            budgets=(1000,),
            seeds=tuple(range(1, 6)),
            optimizers={"random_search": random_search_optimizer},
            include_scipy_de=False,
        )
    out_dir = Path("python/examples/output")
    csv_path = write_benchmark_csv(out_dir / "benchmark_results.csv", summaries)
    md_path = write_benchmark_markdown(out_dir / "benchmark_results.md", summaries)
    print(f"wrote {csv_path}")
    print(f"wrote {md_path}")
    for row in benchmark_table(summaries)[:12]:
        print(row)


if __name__ == "__main__":
    main()
