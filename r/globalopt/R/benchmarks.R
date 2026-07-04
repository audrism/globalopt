benchmark_functions <- function(dim) {
  list(
    list(name = "sphere", lower = rep(-5.12, dim), upper = rep(5.12, dim), optimum = 0,
         fn = function(x) sum(x * x)),
    list(name = "rosenbrock", lower = rep(-2, dim), upper = rep(2, dim), optimum = 0,
         fn = function(x) sum(100 * (x[-1] - x[-length(x)]^2)^2 + (1 - x[-length(x)])^2)),
    list(name = "rastrigin", lower = rep(-5.12, dim), upper = rep(5.12, dim), optimum = 0,
         fn = function(x) 10 * length(x) + sum(x * x - 10 * cos(2 * pi * x))),
    list(name = "ackley", lower = rep(-32.768, dim), upper = rep(32.768, dim), optimum = 0,
         fn = function(x) {
           n <- length(x)
           -20 * exp(-0.2 * sqrt(sum(x * x) / n)) - exp(sum(cos(2 * pi * x)) / n) + 20 + exp(1)
         })
  )
}

benchmark_optimizers <- function() {
  list(
    globalopt_mig2 = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      mig2(lower, upper, budget, fn)$best_f
    },
    globalopt_bayes1 = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      init <- max(2L, min(20L, budget %/% 5L))
      bayes1(lower, upper, budget, init, fn)$best_f
    },
    globalopt_lpmin = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      analysis <- max(0L, min(50L, budget %/% 4L))
      search <- max(1L, budget - analysis)
      lpmin(lower, upper, analysis, search, fn)$best_f
    },
    globalopt_glopt = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      init <- max(10L, min(80L, budget %/% 5L))
      glopt(lower, upper, budget, init, 6L, 0.92, fn)$best_f
    },
    globalopt_unt = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      unt(lower, upper, budget, 0.15, fn)$best_f
    },
    globalopt_exkor = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      x0 <- (lower + upper) / 2
      exkor(x0, lower, upper, max(10L, budget %/% 2L), 0.25, 0.8, fn)$best_f
    },
    globalopt_extr = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      x0 <- (lower + upper) / 2
      extr(x0, lower, upper, budget, fn)$best_f
    },
    globalopt_mivar4 = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      x0 <- (lower + upper) / 2
      mivar4(x0, lower, upper, max(10L, budget %/% 2L), 0.1, fn)$best_f
    },
    globalopt_flexi = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      x0 <- (lower + upper) / 2
      flexi(x0, lower, upper, max(20L, budget), 0.08, fn)$best_f
    },
    globalopt_reqp = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      x0 <- (lower + upper) / 2
      reqp(x0, lower, upper, max(10L, budget %/% 2L), 10, 1.25, fn, function(x) numeric(0))$best_f
    },
    globalopt_lbayes = function(lower, upper, budget, fn, seed) {
      set.seed(seed)
      init <- max(10L, min(80L, budget %/% 5L))
      lit <- max(20L, min(120L, budget %/% 2L))
      lbayes(lower, upper, budget, init, lit, fn)$best_f
    }
  )
}

benchmark_deoptim <- function(lower, upper, budget, fn, seed) {
  if (!requireNamespace("DEoptim", quietly = TRUE)) {
    stop("Package 'DEoptim' is required for benchmark_deoptim()")
  }
  set.seed(seed)
  ctrl <- DEoptim::DEoptim.control(itermax = max(10L, budget %/% max(1L, 20L * length(lower))), trace = FALSE)
  fit <- DEoptim::DEoptim(fn = fn, lower = lower, upper = upper, control = ctrl)
  as.numeric(fit$optim$bestval)
}

benchmark_ga <- function(lower, upper, budget, fn, seed) {
  if (!requireNamespace("GA", quietly = TRUE)) {
    stop("Package 'GA' is required for benchmark_ga()")
  }
  set.seed(seed)
  maxiter <- max(10L, budget %/% max(1L, 10L * length(lower)))
  popsize <- max(20L, min(80L, budget %/% max(1L, maxiter)))
  fit <- GA::ga(
    type = "real-valued",
    fitness = function(x) -as.numeric(fn(x)),
    lower = lower,
    upper = upper,
    popSize = popsize,
    maxiter = maxiter,
    run = max(5L, maxiter %/% 2L),
    monitor = FALSE,
    seed = seed
  )
  -as.numeric(fit@fitnessValue)
}

benchmark_gensa <- function(lower, upper, budget, fn, seed) {
  if (!requireNamespace("GenSA", quietly = TRUE)) {
    stop("Package 'GenSA' is required for benchmark_gensa()")
  }
  set.seed(seed)
  ctrl <- list(max.call = max(50L, budget), verbose = FALSE)
  fit <- GenSA::GenSA(par = (lower + upper) / 2, fn = fn, lower = lower, upper = upper, control = ctrl)
  as.numeric(fit$value)
}

run_benchmarks <- function(
  dimensions = c(2L, 10L, 30L),
  budgets = c(1000L, 10000L),
  seeds = 1:30,
  optimizers = NULL,
  include_deoptim = FALSE,
  include_ga = FALSE,
  include_gensa = FALSE,
  success_tol = 1e-4
) {
  if (is.null(optimizers)) {
    optimizers <- benchmark_optimizers()
  }
  if (include_deoptim) {
    optimizers$deoptim <- benchmark_deoptim
  }
  if (include_ga) {
    optimizers$ga <- benchmark_ga
  }
  if (include_gensa) {
    optimizers$gensa <- benchmark_gensa
  }

  rows <- list()
  row_idx <- 1L

  for (dim in dimensions) {
    problems <- benchmark_functions(dim)
    for (problem in problems) {
      for (budget in budgets) {
        for (opt_name in names(optimizers)) {
          opt <- optimizers[[opt_name]]
          vals <- numeric(length(seeds))
          times <- numeric(length(seeds))
          succ <- 0L

          for (i in seq_along(seeds)) {
            seed <- seeds[[i]]
            t0 <- proc.time()[[3L]]
            vals[[i]] <- as.numeric(opt(problem$lower, problem$upper, budget, problem$fn, seed))
            times[[i]] <- proc.time()[[3L]] - t0
            if (vals[[i]] <= problem$optimum + success_tol) succ <- succ + 1L
          }

          rows[[row_idx]] <- data.frame(
            optimizer = opt_name,
            problem = problem$name,
            dimension = as.integer(dim),
            budget = as.integer(budget),
            runs = as.integer(length(seeds)),
            median_best = stats::median(vals),
            best_of_runs = min(vals),
            success_rate = succ / max(1, length(seeds)),
            median_seconds = stats::median(times),
            stringsAsFactors = FALSE
          )
          row_idx <- row_idx + 1L
        }
      }
    }
  }

  do.call(rbind, rows)
}

write_benchmark_csv <- function(df, file) {
  utils::write.csv(df, file = file, row.names = FALSE)
  invisible(normalizePath(file, winslash = "/", mustWork = FALSE))
}

write_benchmark_markdown <- function(df, file) {
  cols <- names(df)
  header <- paste0("| ", paste(cols, collapse = " | "), " |")
  sep <- paste0("| ", paste(rep("---", length(cols)), collapse = " | "), " |")
  rows <- apply(df, 1, function(r) paste0("| ", paste(as.character(r), collapse = " | "), " |"))
  lines <- c(header, sep, rows)
  writeLines(lines, con = file)
  invisible(normalizePath(file, winslash = "/", mustWork = FALSE))
}
