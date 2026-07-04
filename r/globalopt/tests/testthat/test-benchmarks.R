test_that("run_benchmarks returns expected shape", {
  midpoint_opt <- function(lower, upper, budget, fn, seed) {
    set.seed(seed)
    x <- (lower + upper) / 2
    as.numeric(fn(x))
  }

  tbl <- run_benchmarks(
    dimensions = c(2L),
    budgets = c(10L),
    seeds = 1:2,
    optimizers = list(midpoint = midpoint_opt),
    include_deoptim = FALSE
  )

  expect_s3_class(tbl, "data.frame")
  expect_equal(nrow(tbl), 4L)
  expect_true(all(tbl$optimizer == "midpoint"))
  expect_true(all(tbl$dimension == 2L))
  expect_true(all(tbl$budget == 10L))
  expect_true(all(tbl$runs == 2L))
})

test_that("benchmark writers create output files", {
  df <- data.frame(
    optimizer = "midpoint",
    problem = "sphere",
    dimension = 2L,
    budget = 10L,
    runs = 2L,
    median_best = 0.0,
    best_of_runs = 0.0,
    success_rate = 1.0,
    median_seconds = 0.001,
    stringsAsFactors = FALSE
  )

  td <- tempdir()
  csv_path <- file.path(td, "bench_test.csv")
  md_path <- file.path(td, "bench_test.md")

  write_benchmark_csv(df, csv_path)
  write_benchmark_markdown(df, md_path)

  expect_true(file.exists(csv_path))
  expect_true(file.exists(md_path))
  expect_match(readLines(md_path, warn = FALSE)[1], "^\\| optimizer ")
})

test_that("translated objective functions are exposed and finite", {
  expect_true(is.finite(furasn(c(0.1, -0.2))))

  expect_true(is.finite(fush5(c(4, 4, 4, 4))))
  expect_true(is.finite(fush7(c(4, 4, 4, 4))))
  expect_true(is.finite(fush10(c(4, 4, 4, 4))))

  expect_true(is.finite(fuhar3(c(0.2, 0.3, 0.4))))
  expect_true(is.finite(fuhar6(c(0.2, 0.3, 0.4, 0.5, 0.6, 0.7))))

  expect_true(is.finite(fubran(c(pi, 2.275))))
  expect_true(is.finite(fugold(c(0, -1))))

  expect_true(is.infinite(fush5(c(1, 2, 3))))
  expect_true(is.infinite(fuhar6(c(1, 2, 3))))
  expect_true(is.infinite(fubran(c(1, 2, 3))))
})
