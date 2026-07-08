# Benchmark problem set - R twin of problems.py.  Definitions, problem ids,
# shift generation and instance semantics are bit-identical to the Python
# harness so results are directly comparable across languages.

.lcg_stream <- function(seed, count) {
  x <- seed %% 2147483647
  if (x <= 0) x <- x + 2147483646
  out <- numeric(count)
  for (i in seq_len(count)) {
    x <- (x * 16807) %% 2147483647
    out[[i]] <- x / 2147483647
  }
  out
}

.p_sphere <- function(x) sum(x * x)
.p_rosenbrock <- function(x) {
  n <- length(x)
  sum(100 * (x[-1] - x[-n]^2)^2 + (1 - x[-n])^2)
}
.p_rastrigin <- function(x) 10 * length(x) + sum(x * x - 10 * cos(2 * pi * x))
.p_ackley <- function(x) {
  n <- length(x)
  -20 * exp(-0.2 * sqrt(sum(x * x) / n)) - exp(sum(cos(2 * pi * x)) / n) + 20 + exp(1)
}
.p_griewank <- function(x) {
  sum(x * x) / 4000 - prod(cos(x / sqrt(seq_along(x)))) + 1
}
.p_levy <- function(x) {
  w <- 1 + (x - 1) / 4
  n <- length(w)
  t1 <- sin(pi * w[[1]])^2
  t3 <- (w[[n]] - 1)^2 * (1 + sin(2 * pi * w[[n]])^2)
  t2 <- if (n > 1) sum((w[-n] - 1)^2 * (1 + 10 * sin(pi * w[-n] + 1)^2)) else 0
  t1 + t2 + t3
}
.p_schwefel <- function(x) {
  418.9829101183649 * length(x) - sum(x * sin(sqrt(abs(x))))
}
.p_zakharov <- function(x) {
  s1 <- sum(x * x)
  s2 <- sum(0.5 * seq_along(x) * x)
  s1 + s2^2 + s2^4
}

.p_branin <- function(x) {
  x1 <- x[[1]]; x2 <- x[[2]]
  b <- 5.1 / (4 * pi^2); c <- 5 / pi; t <- 1 / (8 * pi)
  (x2 - b * x1 * x1 + c * x1 - 6)^2 + 10 * (1 - t) * cos(x1) + 10
}
.p_goldstein <- function(x) {
  x1 <- x[[1]]; x2 <- x[[2]]
  t1 <- 1 + (x1 + x2 + 1)^2 * (19 - 14 * x1 + 3 * x1 * x1 - 14 * x2 + 6 * x1 * x2 + 3 * x2 * x2)
  t2 <- 30 + (2 * x1 - 3 * x2)^2 * (18 - 32 * x1 + 12 * x1 * x1 + 48 * x2 - 36 * x1 * x2 + 27 * x2 * x2)
  t1 * t2
}
.p_camel <- function(x) {
  x1 <- x[[1]]; x2 <- x[[2]]
  (4 - 2.1 * x1 * x1 + x1^4 / 3) * x1 * x1 + x1 * x2 + (-4 + 4 * x2 * x2) * x2 * x2
}

.SHEK_A <- matrix(c(
  4, 4, 4, 4,
  1, 1, 1, 1,
  8, 8, 8, 8,
  6, 6, 6, 6,
  3, 7, 3, 7,
  2, 9, 2, 9,
  5, 5, 3, 3,
  8, 1, 8, 1,
  6, 2, 6, 2,
  7, 3.6, 7, 3.6
), nrow = 10, byrow = TRUE)
.SHEK_C <- c(0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3, 0.7, 0.5, 0.5)

.p_shekel <- function(x, m) {
  f <- 0
  for (i in seq_len(m)) {
    d <- .SHEK_C[[i]] + sum((x - .SHEK_A[i, ])^2)
    f <- f - 1 / d
  }
  f
}

.H_C <- c(1, 1.2, 3, 3.2)
.H3_ALPHA <- matrix(c(3, 10, 30, 0.1, 10, 35, 3, 10, 30, 0.1, 10, 35),
                    nrow = 4, byrow = TRUE)
.H3_P <- matrix(c(
  0.3689, 0.1170, 0.2673,
  0.4699, 0.4387, 0.7470,
  0.1091, 0.8732, 0.5547,
  0.03815, 0.5743, 0.8828
), nrow = 4, byrow = TRUE)
.H6_ALPHA <- matrix(c(
  10, 3, 17, 3.5, 1.7, 8,
  0.05, 10, 17, 0.1, 8, 14,
  3, 3.5, 1.7, 10, 17, 8,
  17, 8, 0.05, 10, 0.1, 14
), nrow = 4, byrow = TRUE)
.H6_P <- matrix(c(
  0.1312, 0.1696, 0.5569, 0.0124, 0.8283, 0.5886,
  0.2329, 0.4135, 0.8307, 0.3736, 0.1004, 0.9991,
  0.2348, 0.1451, 0.3522, 0.2883, 0.3047, 0.6650,
  0.4047, 0.8828, 0.8732, 0.5743, 0.1091, 0.0381
), nrow = 4, byrow = TRUE)

.p_hartmann <- function(x, alpha, p) {
  f <- 0
  for (i in 1:4) {
    f <- f - .H_C[[i]] * exp(-sum(alpha[i, ] * (x - p[i, ])^2))
  }
  f
}

.SCALABLE <- list(
  sphere = list(fn = .p_sphere, lo = -5.12, hi = 5.12, fopt = 0),
  rosenbrock = list(fn = .p_rosenbrock, lo = -2.048, hi = 2.048, fopt = 0),
  rastrigin = list(fn = .p_rastrigin, lo = -5.12, hi = 5.12, fopt = 0),
  ackley = list(fn = .p_ackley, lo = -32.768, hi = 32.768, fopt = 0),
  griewank = list(fn = .p_griewank, lo = -600, hi = 600, fopt = 0),
  levy = list(fn = .p_levy, lo = -10, hi = 10, fopt = 0),
  schwefel = list(fn = .p_schwefel, lo = -500, hi = 500, fopt = 0),
  zakharov = list(fn = .p_zakharov, lo = -5, hi = 10, fopt = 0)
)

.FIXED <- list(
  branin = list(fn = .p_branin, lower = c(-5, 0), upper = c(10, 15),
                fopt = 0.39788735772973816),
  goldstein_price = list(fn = .p_goldstein, lower = c(-2, -2), upper = c(2, 2),
                         fopt = 3.0),
  six_hump_camel = list(fn = .p_camel, lower = c(-3, -2), upper = c(3, 2),
                        fopt = -1.0316284534898774),
  shekel5 = list(fn = function(x) .p_shekel(x, 5), lower = rep(0, 4),
                 upper = rep(10, 4), fopt = -10.153199679058231),
  shekel7 = list(fn = function(x) .p_shekel(x, 7), lower = rep(0, 4),
                 upper = rep(10, 4), fopt = -10.402940566818664),
  shekel10 = list(fn = function(x) .p_shekel(x, 10), lower = rep(0, 4),
                  upper = rep(10, 4), fopt = -10.536409816692023),
  hartmann3 = list(fn = function(x) .p_hartmann(x, .H3_ALPHA, .H3_P),
                   lower = rep(0, 3), upper = rep(1, 3),
                   fopt = -3.8627821478207558),
  hartmann6 = list(fn = function(x) .p_hartmann(x, .H6_ALPHA, .H6_P),
                   lower = rep(0, 6), upper = rep(1, 6),
                   fopt = -3.3223680113913385)
)

SCALABLE_DIMS <- c(2L, 5L, 10L)
SHIFT_FRACTION <- 0.05
PROBLEM_IDS <- setNames(seq_along(c(names(.SCALABLE), names(.FIXED))),
                        c(names(.SCALABLE), names(.FIXED)))

make_problem <- function(name, dim, instance) {
  if (!is.null(.SCALABLE[[name]])) {
    spec <- .SCALABLE[[name]]
    lower <- rep(spec$lo, dim)
    upper <- rep(spec$hi, dim)
    fn <- spec$fn
    fopt <- spec$fopt
  } else {
    spec <- .FIXED[[name]]
    lower <- spec$lower
    upper <- spec$upper
    fn <- spec$fn
    fopt <- spec$fopt
    if (dim != length(lower)) stop(name, " is fixed at dimension ", length(lower))
  }
  n <- length(lower)
  delta <- rep(0, n)
  if (instance > 0) {
    seed <- PROBLEM_IDS[[name]] * 10000 + n * 100 + instance
    u <- .lcg_stream(seed, n)
    delta <- (2 * u - 1) * SHIFT_FRACTION * (upper - lower)
  }
  objective <- function(x) fn(x - delta) - fopt
  list(name = name, dim = n, instance = instance, lower = lower,
       upper = upper, objective = objective)
}

default_problem_names <- function() {
  scalable <- expand.grid(name = names(.SCALABLE), dim = SCALABLE_DIMS,
                          stringsAsFactors = FALSE)
  fixed <- data.frame(name = names(.FIXED),
                      dim = vapply(.FIXED, function(s) length(s$lower), 1L))
  rbind(scalable, fixed)
}
