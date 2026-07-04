# Apples-to-Apples (Globalopt-only)

This comparison uses the same translated globalopt method set and same benchmark settings in both interfaces:

- Methods: globalopt_mig2, globalopt_bayes1, globalopt_lpmin, globalopt_glopt, globalopt_unt, globalopt_exkor, globalopt_extr, globalopt_mivar4, globalopt_flexi, globalopt_reqp, globalopt_lbayes
- Problems: sphere, rosenbrock, rastrigin, ackley (dimension 2)
- Budget: 120
- Seeds: 1..3

| problem | python_winner | python_gap | python_time_s | r_winner | r_gap | r_time_s |
| --- | --- | --- | --- | --- | --- | --- |
| ackley | globalopt_glopt | 4.44089e-16 | 0.000208209 | globalopt_exkor | 4.44089e-16 | 0.001 |
| rastrigin | globalopt_extr | 0 | 0.000152811 | globalopt_exkor | 0 | 0.001 |
| rosenbrock | globalopt_flexi | 1.44784e-22 | 0.000574778 | globalopt_lbayes | 1.75599e-08 | 0.561 |
| sphere | globalopt_glopt | 0 | 0.00010963 | globalopt_mivar4 | 0 | 0.001 |

## Data Files

- docs/benchmarks/apples_to_apples_python_globalopt.csv
- docs/benchmarks/apples_to_apples_r_globalopt.csv
