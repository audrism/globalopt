/* Public interface of the GlobalMinimum C shim (gm_shim.c) and the
 * Fortran entry points wrapped by the R and Python packages. */
#ifndef GM_SHIM_H
#define GM_SHIM_H

#ifdef __cplusplus
extern "C" {
#endif

typedef double (*gm_objective_fn)(const double *x, int n, void *data);
typedef void (*gm_constraint_fn)(const double *x, int nx, double *r, int k8,
                                 void *data);

void gm_set_objective(gm_objective_fn f, void *data);
int gm_set_builtin(int idx);
int gm_n_builtins(void);
void gm_set_constraints(gm_constraint_fn f, void *data);

void gm_reset_evals(void);
long gm_get_evals(void);

int gm_trace_start(long cap, int dim);
long gm_trace_length(void);
int gm_trace_dimension(void);
const double *gm_trace_points(void);
const double *gm_trace_values(void);
void gm_trace_stop(void);

void gm_ats_set(const double *state15);
void gm_ats_get(double *state15);
void gm_ats_reset(void);

/* Built-in objective indices (order of GM_BUILTINS in gm_shim.c) */
#define GM_BUILTIN_FURASN 0
#define GM_BUILTIN_FUSH5 1
#define GM_BUILTIN_FUSH7 2
#define GM_BUILTIN_FUSH10 3
#define GM_BUILTIN_FUHAR3 4
#define GM_BUILTIN_FUHAR6 5
#define GM_BUILTIN_FUBRAN 6
#define GM_BUILTIN_FUGOLD 7

/* Fortran optimizer entry points (gfortran name mangling). */
void mig1_(double *x, const double *a, const double *b, const int *n,
           double *fm, const int *ipar, const int *ipa);
void mig2_(double *x, const double *a, const double *b, const int *n,
           double *xn, const int *nm, double *fm, const int *ipar,
           const int *ipa);
void bayes1_(double *x, const double *a, const double *b, const int *n,
             double *xn, const int *nm, double *fm, const int *ipar,
             const int *ipa);
void lbayes_(double *x, const double *a, const double *b, const int *n,
             double *f1, const int *ipar, const double *par, const int *ipa,
             const int *ipaa);
void unt_(double *x, const double *a, const double *b, const int *n,
          double *xn, const int *nm, double *fm, const int *ipar,
          const int *ipa);
void extr_(double *xm, const double *bp, const double *ep, double *ym,
           const int *ipar, const double *par, const int *ipa,
           const int *ipaa);
void exkor_(double *x, const double *a, const double *b, const int *n,
            double *fm, const int *ipar, const double *par, const int *ipa,
            const int *ipaa);
void mivar4_(double *x, const double *a, const double *b, const int *nn,
             double *b1, const int *nm, double *fm, const int *ipar,
             const double *par, const int *ipa, const int *ipaa);
void flexi_(double *z, const int *m1, double *ff, const int *ipar,
            const double *par, const int *ipa, const int *ipaa);
void glopt_(double *xm, const double *a, const double *b, const int *m,
            double *fm, const int *ipar, const int *ipa);
void lpmin_(double *x, const double *a, const double *b, const int *n,
            double *x2, const int *nm, double *fmin, const int *ipar,
            const int *ipa);
void anal1_(const double *xp, const double *xg, const int *n, double *xx,
            double *x, const int *nm, const int *ipar, const int *ipa);
void anal2_(const double *a, const double *b, const int *n, double *xx,
            double *x, const int *nm, const int *ipar, const int *ipa);
void reqp_(double *x, double *b1, double *q, double *a, const int *n,
           double *fm, const int *ipar, const double *par, const int *ipa,
           const int *ipaa);

/* Fortran utilities */
void lptau_(const double *c, const int *n, double *x);
double furasn_(const double *x, const int *n);

#ifdef __cplusplus
}
#endif

#endif /* GM_SHIM_H */
