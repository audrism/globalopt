if (!requireNamespace("testthat", quietly = TRUE)) {
	quit("no", status = 0)
}

library(testthat)
library(globalopt)

test_check("globalopt")
