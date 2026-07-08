# Reference values in these tests were produced by a standalone C driver
# linked directly against the Fortran library (see
# docs/FORTRAN_INTERFACES.md in the source repository); they pin the
# package to fresh-process Fortran behavior.

A2 <- c(-0.25, -0.125)
B2 <- c(0.5, 0.625)

test_that("lp_tau_point matches the canonical sequence and the reference backend", {
  expect_equal(lp_tau_point(1, 2), c(0.5, 0.5))
  expect_equal(lp_tau_point(2, 2), c(0.25, 0.75))
  expect_equal(lp_tau_point(3, 2), c(0.75, 0.25))
  expect_equal(lp_tau_point(4, 2), c(0.125, 0.625))
  for (k in c(1, 5, 17, 100)) {
    expect_equal(lp_tau_point(k, 6), lp_tau_point(k, 6, backend = "reference"),
                 tolerance = 1e-12)
  }
})

test_that("compiled built-in objectives match the R implementations", {
  x <- c(0.2, -0.1)
  expect_equal(eval_builtin("furasn", x), furasn(x), tolerance = 1e-12)
  expect_equal(eval_builtin("fubran", x), fubran(x), tolerance = 1e-12)
  expect_equal(eval_builtin("fugold", x), fugold(x), tolerance = 1e-12)
  x4 <- c(4, 4, 4, 4)
  expect_equal(eval_builtin("fush5", x4), fush5(x4), tolerance = 1e-12)
  expect_equal(eval_builtin("fush7", x4), fush7(x4), tolerance = 1e-12)
  expect_equal(eval_builtin("fush10", x4), fush10(x4), tolerance = 1e-12)
  x3 <- c(0.1, 0.5, 0.5)
  expect_equal(eval_builtin("fuhar3", x3), fuhar3(x3), tolerance = 1e-12)
  x6 <- rep(0.2, 6)
  expect_equal(eval_builtin("fuhar6", x6), fuhar6(x6), tolerance = 1e-12)
})

test_that("bayes1 reproduces the fresh-process Fortran run", {
  r <- bayes1(A2, B2, 200, 20, "furasn")
  expect_equal(r$best_f, -1.9966149827762463, tolerance = 1e-12)
  expect_equal(as.integer(r$evals), 200L)
  expect_equal(r$values[1:3],
               c(0.91709442215351888, -0.21484719829221782, -1.1207175656694048),
               tolerance = 1e-12)
  expect_equal(nrow(r$points), 200L)
  expect_equal(r$status, 0L)
})

test_that("mig2 reproduces the fresh-process Fortran run", {
  r <- mig2(A2, B2, 200, "furasn")
  expect_equal(r$best_f, -1.8296951132230181, tolerance = 1e-12)
  expect_equal(as.integer(r$evals), 200L)
})

test_that("an R objective gives the identical trajectory to the built-in", {
  r1 <- bayes1(A2, B2, 100, 20, "furasn")
  r2 <- bayes1(A2, B2, 100, 20, furasn)
  expect_equal(r1$best_f, r2$best_f, tolerance = 1e-12)
  expect_equal(r1$best_x, r2$best_x, tolerance = 1e-12)
  expect_equal(r1$values, r2$values, tolerance = 1e-12)
})

test_that("seeding perturbs and reproduces stochastic runs", {
  r1 <- mig2(A2, B2, 100, "furasn", seed = 1)
  r2 <- mig2(A2, B2, 100, "furasn", seed = 1)
  r3 <- mig2(A2, B2, 100, "furasn", seed = 2)
  expect_equal(r1$best_f, r2$best_f, tolerance = 0)
  expect_false(isTRUE(all.equal(r1$values, r3$values)))
})

test_that("runs are isolated: a run does not perturb the next one", {
  r1 <- mig2(A2, B2, 100, "furasn")
  invisible(mig2(A2, B2, 57, "furasn"))
  r2 <- mig2(A2, B2, 100, "furasn")
  expect_equal(r1$best_f, r2$best_f, tolerance = 0)
})

test_that("objective errors propagate as R errors", {
  expect_error(
    mig2(A2, B2, 50, function(x) stop("boom")),
    "signalled an error"
  )
})

test_that("input validation catches upstream limits", {
  expect_error(mig2(A2, B2, 2000, "furasn"), "1..1000")
  expect_error(bayes1(A2, B2, 100, 200, "furasn"), "initial_points")
  expect_error(mig2(rep(0, 21), rep(1, 21), 100, "furasn"), "dimension")
  expect_error(mig2(c(1, 0), c(0, 1), 10, "furasn"), "lower bounds")
  expect_error(mig2(A2, B2, 10, "nosuch"), "unknown built-in")
})

test_that("the remaining optimizers run and improve on the start value", {
  f0 <- eval_builtin("furasn", (A2 + B2) / 2)

  r <- mig1(A2, B2, 200, "furasn")
  expect_lt(r$best_f, f0)

  r <- lbayes(A2, B2, iterations = 20, objective = "furasn")
  expect_lt(r$best_f, f0)

  r <- unt(A2, B2, 200, objective = "furasn")
  expect_lt(r$best_f, f0)

  r <- glopt(A2, B2, 500, objective = "furasn")
  expect_lt(r$best_f, f0)

  r <- lpmin(A2, B2, 50, 100, objective = "furasn")
  expect_lt(r$best_f, f0)

  r <- exkor(A2, B2, 100, x0 = c(0.1, 0.1), objective = "furasn")
  expect_lt(r$best_f, f0)

  r <- mivar4(A2, B2, 100, x0 = c(0.2, 0.2), objective = "furasn")
  expect_lte(r$best_f, eval_builtin("furasn", c(0.2, 0.2)))

  r <- flexi(c(0.2, 0.2), 200, objective = "furasn")
  expect_lte(r$best_f, eval_builtin("furasn", c(0.2, 0.2)))

  r <- reqp(c(0.2, 0.2), 50, objective = "furasn")
  expect_lte(r$best_f, eval_builtin("furasn", c(0.2, 0.2)))
})

test_that("extr solves a smooth 1-D problem", {
  r <- extr(-1, 1, 50, objective = function(x) (x[1] - 0.3)^2)
  expect_lt(abs(r$best_x - 0.3), 0.05)
  expect_lt(r$best_f, 1e-3)
})

test_that("flexi and reqp honor inequality constraints", {
  # minimize furasn subject to x1 >= 0.05 (inequality g = x1 - 0.05 >= 0)
  con <- function(x) x[1] - 0.05
  r <- flexi(c(0.2, 0.2), 300, constraints = con, n_ineq = 1L,
             objective = "furasn")
  expect_gte(r$best_x[1], 0.05 - 1e-3)
  r <- reqp(c(0.2, 0.2), 50, constraints = con, n_ineq = 1L,
            objective = "furasn")
  expect_gte(r$best_x[1], 0.05 - 1e-3)
})

test_that("analysis routines process a design", {
  r <- mig2(c(-1, -1), c(1, 1), 100, "furasn")
  a1 <- anal1(c(-1, -1), c(1, 1), r$points, r$values)
  expect_true(length(a1$influence) >= 1)
  expect_true(all(diff(a1$influence) <= 1e-8))
  a2 <- anal2(c(-1, -1), c(1, 1), r$points, r$values)
  expect_length(a2$influence, 2L)
  expect_length(a2$influence_eigen, 2L)
  expect_equal(dim(a2$eigenvectors), c(2L, 2L))
})

test_that("reference backend agrees with fortran on the initial design", {
  rf <- bayes1(A2, B2, 20, 20, "furasn")
  rr <- bayes1(A2, B2, 20, 20, furasn, backend = "reference")
  # identical LP-tau initial points => identical values
  expect_equal(rf$values, rr$values, tolerance = 1e-10)
})

test_that("print method renders", {
  r <- mig1(A2, B2, 50, "furasn")
  expect_output(print(r), "mig1")
})
