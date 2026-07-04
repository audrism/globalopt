# Winner Tables

Generated from the latest benchmark CSV files in this repository.

## Python Per-Problem Winners

| dimension | problem | budget | winner | family | winner_gap | runner_up | runner_up_gap | margin_to_2nd |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | ackley | 120 | globalopt_glopt | globalopt | 4.44089e-16 | globalopt_extr | 4.44089e-16 | 0 |
| 2 | griewank | 120 | globalopt_glopt | globalopt | 0 | globalopt_extr | 0 | 0 |
| 2 | levy | 120 | globalopt_flexi | globalopt | 2.50381e-24 | globalopt_lbayes | 1.35613e-11 | 1.35613e-11 |
| 2 | rastrigin | 120 | globalopt_glopt | globalopt | 0 | globalopt_extr | 0 | 0 |
| 2 | rosenbrock | 120 | globalopt_flexi | globalopt | 1.44784e-22 | globalopt_lbayes | 1.11833e-08 | 1.11833e-08 |
| 2 | sphere | 120 | globalopt_glopt | globalopt | 0 | globalopt_extr | 0 | 0 |

Summary: globalopt wins = 6, external wins = 0, total groups = 6.

## R Per-Problem Winners

| dimension | problem | budget | winner | family | winner_gap | runner_up | runner_up_gap | margin_to_2nd |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | ackley | 120 | gensa | external | 4.44089e-16 | globalopt_exkor | 4.44089e-16 | 0 |
| 2 | rastrigin | 120 | gensa | external | 0 | globalopt_exkor | 0 | 0 |
| 2 | rosenbrock | 120 | gensa | external | 4.11841e-18 | globalopt_lbayes | 1.75599e-08 | 1.75599e-08 |
| 2 | sphere | 120 | globalopt_extr | globalopt | 0 | globalopt_exkor | 0 | 0 |

Summary: globalopt wins = 1, external wins = 3, total groups = 4.

