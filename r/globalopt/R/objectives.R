#' Built-in test objectives (R implementations)
#'
#' R implementations of the test objectives shipped with the original
#' GlobalMinimum library: `furasn` (a rastrigin-like separable function,
#' global minimum -2 at the origin), the Shekel family `fush5`/`fush7`/
#' `fush10` (4-D), the Hartmann family `fuhar3` (3-D) and `fuhar6` (6-D),
#' `fubran` (Branin, 2-D) and `fugold` (Goldstein-Price, 2-D).
#' Identical compiled versions are available by passing the function name
#' as the `objective` argument of the optimizers (see
#' [builtin_objectives()]).
#'
#' @param x Numeric vector.
#' @return Objective value (numeric scalar).
#' @name objectives
NULL

#' @rdname objectives
#' @export
furasn <- function(x) {
  if (length(x) == 0L) {
    return(0)
  }
  sum((x * x) - cos(18 * x)) * (2 / length(x))
}

#' @rdname objectives
#' @export
fush5 <- function(x) {
  .shekel_family(x, .SHEKEL_A5, .SHEKEL_C5)
}

#' @rdname objectives
#' @export
fush7 <- function(x) {
  .shekel_family(x, .SHEKEL_A7, .SHEKEL_C7)
}

#' @rdname objectives
#' @export
fush10 <- function(x) {
  .shekel_family(x, .SHEKEL_A10, .SHEKEL_C10)
}

#' @rdname objectives
#' @export
fuhar3 <- function(x) {
  .hartmann_family(x, .HAR3_ALPHA, .HAR_C, .HAR3_P)
}

#' @rdname objectives
#' @export
fuhar6 <- function(x) {
  .hartmann_family(x, .HAR6_ALPHA, .HAR_C, .HAR6_P)
}

#' @rdname objectives
#' @export
fubran <- function(x) {
  if (length(x) != 2L) {
    return(Inf)
  }
  x1 <- x[[1L]]
  x2 <- x[[2L]]
  (x2 - 0.1292 * x1 * x1 + 1.59155 * x1 - 6)^2 + 9.60211 * cos(x1) + 10
}

#' @rdname objectives
#' @export
fugold <- function(x) {
  if (length(x) != 2L) {
    return(Inf)
  }
  x1 <- x[[1L]]
  x2 <- x[[2L]]
  x3 <- x1 * x1
  x4 <- x2 * x2
  x5 <- x1 * x2
  (1 + (x1 + x2 + 1)^2 * (19 - 14 * x1 + 3 * x3 - 14 * x2 + 6 * x5 + 3 * x4)) *
    (30 + (2 * x1 - 3 * x2)^2 * (18 - 32 * x1 + 12 * x3 + 48 * x2 - 36 * x5 + 27 * x4))
}
.shekel_family <- function(x, a, c) {
  if (length(x) != 4L) {
    return(Inf)
  }
  f <- 0
  for (i in seq_len(nrow(a))) {
    f1 <- c[[i]]
    for (j in seq_len(length(x))) {
      d <- x[[j]] - a[i, j]
      f1 <- f1 + d * d
    }
    f <- f - 1 / f1
  }
  f
}

.hartmann_family <- function(x, alpha, c, p) {
  if (length(x) != ncol(alpha)) {
    return(Inf)
  }
  f <- 0
  for (i in seq_len(nrow(alpha))) {
    f1 <- 0
    for (j in seq_len(length(x))) {
      d <- x[[j]] - p[i, j]
      f1 <- f1 - alpha[i, j] * d * d
    }
    f <- f - c[[i]] * exp(f1)
  }
  f
}

.SHEKEL_A5 <- matrix(
  c(
    4, 1, 8, 6,
    3, 4, 1, 8,
    6, 7, 4, 1,
    8, 6, 3, 4,
    1, 8, 6, 7
  ),
  nrow = 5,
  byrow = FALSE
)

.SHEKEL_C5 <- c(0.1, 0.2, 0.2, 0.4, 0.4)

.SHEKEL_A7 <- matrix(
  c(
    4, 1, 8, 6,
    3, 2, 5, 4,
    1, 8, 6, 7,
    9, 5, 4, 1,
    8, 6, 3, 2,
    3, 4, 1, 8,
    6, 7, 9, 3
  ),
  nrow = 7,
  byrow = FALSE
)

.SHEKEL_C7 <- c(0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3)

.SHEKEL_A10 <- matrix(
  c(
    4, 1, 8, 6,
    3, 2, 5, 8,
    6, 7, 4, 1,
    8, 6, 7, 9,
    5, 1, 2, 3.6,
    4, 1, 8, 6,
    3, 2, 3, 8,
    6, 7, 4, 1,
    8, 6, 7, 9,
    3, 1, 2, 3.6
  ),
  nrow = 10,
  byrow = FALSE
)

.SHEKEL_C10 <- c(0.1, 0.2, 0.2, 0.4, 0.4, 0.6, 0.3, 0.7, 0.5, 0.5)

.HAR3_ALPHA <- matrix(
  c(
    3, 0.1, 3,
    0.1, 10, 10,
    10, 10, 30,
    35, 30, 35
  ),
  nrow = 4,
  byrow = FALSE
)

.HAR3_P <- matrix(
  c(
    0.3689, 0.4699, 0.1091,
    0.03815, 0.117, 0.4387,
    0.8732, 0.5743, 0.2673,
    0.7470, 0.5547, 0.8828
  ),
  nrow = 4,
  byrow = FALSE
)

.HAR6_ALPHA <- matrix(
  c(
    10, 0.05, 3, 17, 3, 10,
    3.5, 8, 17, 17, 1.7, 0.05,
    3.5, 0.1, 10, 10, 1.7, 8,
    17, 0.1, 8, 14, 8, 14
  ),
  nrow = 4,
  byrow = FALSE
)

.HAR6_P <- matrix(
  c(
    0.1312, 0.2329, 0.2348, 0.4047, 0.1696, 0.4135,
    0.1451, 0.8828, 0.5569, 0.8307, 0.3522, 0.8732,
    0.0124, 0.3736, 0.2883, 0.5743, 0.8283, 0.1004,
    0.3047, 0.1091, 0.5886, 0.9991, 0.6650, 0.0381
  ),
  nrow = 4,
  byrow = FALSE
)

.HAR_C <- c(1, 1.2, 3, 3.2)
