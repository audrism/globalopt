# globalopt 0.2.0

* The package now compiles the original GlobalMinimum Fortran library
  (Jonas Mockus, 1989) and runs it by default (`backend = "fortran"`).
  The previous pure-R implementations remain available as
  `backend = "reference"` for the core methods.
* New functions: `mig1()`, `builtin_objectives()`, `eval_builtin()`,
  `ats_state()`, `set_ats_state()`.
* `extr()` is now the original one-dimensional Wiener-model method;
  `exkor()` the original coordinate version; `unt()`, `glopt()`,
  `lpmin()`, `mivar4()`, `flexi()`, `reqp()`, `lbayes()`, `anal1()` and
  `anal2()` now expose the genuine 1989 algorithms with their original
  parameters (signatures changed accordingly).
* Objectives can be passed as the name of a compiled built-in
  (`"furasn"`, `"fush5"`, ..., `"fugold"`), avoiding R callbacks entirely.
* All optimizers gained `trace` (full evaluation record) and `seed`
  (ATS generator state) arguments; runs are deterministic by default and
  reproduce a fresh-process run of the original code.
* Fixed: the R implementations of the Shekel (`fush5`, `fush7`, `fush10`)
  and Hartmann (`fuhar3`, `fuhar6`) test functions had transposed
  coefficient matrices relative to the original Fortran (and the
  literature).
* Results are returned as class `globalopt_result` with a print method
  and a `status` field carrying the Fortran termination code.
* Removed the in-package benchmark helpers; benchmarking now lives in the
  source repository.

# globalopt 0.1.0

* Initial pure-R translation.
