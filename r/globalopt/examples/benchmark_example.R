library(globalopt)

out_dir <- file.path("r", "globalopt", "examples", "output")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

tbl <- run_benchmarks(
  dimensions = c(2L, 10L),
  budgets = c(1000L),
  seeds = 1:5
)

csv_path <- file.path(out_dir, "benchmark_results.csv")
md_path <- file.path(out_dir, "benchmark_results.md")

write_benchmark_csv(tbl, csv_path)
write_benchmark_markdown(tbl, md_path)

cat("wrote", csv_path, "\n")
cat("wrote", md_path, "\n")
print(head(tbl, 12L))
