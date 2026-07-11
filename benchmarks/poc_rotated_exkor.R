#!/usr/bin/env Rscript
# Proof-of-concept for a compound 1989 algorithm: use ANAL2's estimated
# eigenframe to rotate the coordinate system, then run EXKOR in the
# rotated frame.  Tests whether the library's own screening tool can
# repair its own coordinate method's non-separability blindness.
#
# Protocol: for each problem, spend 100 evaluations on an LP-tau design
# for ANAL2 (counted against the budget), then run EXKOR on
# g(y) = f(x_c + V y) where V is the eigenvector matrix and x_c the
# best design point, with the y-box chosen conservatively so x stays in
# the original box (clipped otherwise).  Compare against plain EXKOR
# with the same total budget on gap-to-optimum over seeded instances.

suppressMessages(library(globalopt))
script_dir <- dirname(normalizePath(sub("^--file=", "", grep("^--file=", commandArgs(FALSE), value = TRUE)[1])))
source(file.path(script_dir, "problems.R"))

rotated_exkor <- function(prob, budget, design_evals = 100L) {
  lo <- prob$lower; hi <- prob$upper
  n <- length(lo)
  evals <- 0L
  f_count <- function(x) { evals <<- evals + 1L; prob$objective(x) }

  # 1. LP-tau design + ANAL2 eigenframe
  m <- min(design_evals, 300L)
  pts <- t(vapply(seq_len(m), function(k) lo + lp_tau_point(k, n) * (hi - lo),
                  numeric(n)))
  vals <- apply(pts, 1, f_count)
  a2 <- anal2(lo, hi, pts, vals)
  V <- a2$eigenvectors
  xc <- pts[which.min(vals), ]

  # adaptive rule: rotate only when the eigenframe carries real signal,
  # i.e. its dominant direction explains markedly more influence than
  # the best original axis.  On separable or isotropic problems the two
  # match (or the frame is arbitrary), and rotating would destroy
  # separability, so keep the original axes.
  if (max(a2$influence_eigen) <= 1.3 * max(a2$influence)) {
    V <- diag(n)
  }

  # 2. EXKOR in the rotated frame around the best design point
  half <- (hi - lo) / 2
  ybox <- rep(sqrt(sum(half^2)) / sqrt(n), n)  # conservative radius
  g <- function(y) {
    x <- xc + as.numeric(V %*% y)
    f_count(pmin(pmax(x, lo), hi))
  }
  per_coord <- max(6L, min(500L, (budget - m) %/% (2L * n)))
  r <- exkor(-ybox, ybox, per_coord, x0 = rep(0, n), objective = g,
             trace = FALSE)
  list(best_f = min(r$best_f, min(vals)), evals = evals)
}

plain_exkor <- function(prob, budget) {
  n <- length(prob$lower)
  per_coord <- max(6L, min(500L, budget %/% (2L * n)))
  r <- exkor(prob$lower, prob$upper, per_coord,
             x0 = (prob$lower + prob$upper) / 2,
             objective = prob$objective, trace = FALSE)
  r$best_f
}

# rotated ellipsoid: EXKOR's canonical failure mode
make_rotated_ellipsoid <- function(n, instance) {
  set.seed(instance)
  M <- matrix(rnorm(n * n), n)
  Q <- qr.Q(qr(M))
  u <- .lcg_stream(9990000L + n * 100L + instance, n)
  delta <- (2 * u - 1) * 0.05 * 10
  list(name = "rotated_ellipsoid", dim = n, instance = instance,
       lower = rep(-5, n), upper = rep(5, n),
       objective = function(x) {
         z <- as.numeric(Q %*% (x - delta))
         sum(10^(6 * (seq_len(n) - 1) / (n - 1)) * z^2)
       })
}

cat("problem dim instance plain rotated\n")
res <- list()
for (spec in list(c("zakharov", 5), c("rosenbrock", 5), c("rastrigin", 5))) {
  for (inst in 1:10) {
    prob <- make_problem(spec[1], as.integer(spec[2]), inst)
    budget <- 100L * as.integer(spec[2])
    p <- plain_exkor(prob, budget)
    r <- rotated_exkor(prob, budget)
    res[[length(res) + 1]] <- data.frame(problem = spec[1], dim = spec[2],
                                         instance = inst, plain = p,
                                         rotated = r$best_f)
  }
}
for (inst in 1:10) {
  prob <- make_rotated_ellipsoid(5L, inst)
  p <- plain_exkor(prob, 500L)
  r <- rotated_exkor(prob, 500L)
  res[[length(res) + 1]] <- data.frame(problem = "rotated_ellipsoid",
                                       dim = 5, instance = inst,
                                       plain = p, rotated = r$best_f)
}
df <- do.call(rbind, res)
write.csv(df, "benchmarks/results/poc_rotated_exkor.csv", row.names = FALSE)
agg <- aggregate(cbind(plain, rotated) ~ problem, df, median)
agg$improvement <- agg$plain / agg$rotated
print(agg, digits = 3)
