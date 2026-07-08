/* Language-agnostic C shim between the GlobalMinimum Fortran library and
 * host-language wrappers (R, Python, Rust).
 *
 * The Fortran trampoline FI(X,N) calls gmcall_(x,&n); CONSTR calls
 * gmcon_(...).  This shim dispatches those calls either to a registered
 * host callback or to one of the compiled built-in objectives, counts
 * evaluations, and optionally records the full evaluation trace.
 *
 * All state is global: the upstream library itself is stateful (COMMON
 * blocks), so calls must be serialized by the host anyway.
 */

#include <stdlib.h>
#include <string.h>

typedef double (*gm_objective_fn)(const double *x, int n, void *data);
typedef void (*gm_constraint_fn)(const double *x, int nx, double *r, int k8,
                                 void *data);

/* Fortran built-in objectives from gm_util.f */
extern double furasn_(const double *x, const int *n);
extern double fush5_(const double *x, const int *n);
extern double fush7_(const double *x, const int *n);
extern double fush10_(const double *x, const int *n);
extern double fuhar3_(const double *x, const int *n);
extern double fuhar6_(const double *x, const int *n);
extern double fubran_(const double *x, const int *n);
extern double fugold_(const double *x, const int *n);

/* ATS state setters from gm_fi.f */
extern void gmatse_(const double *s);
extern void gmatge_(double *s);

typedef double (*gm_fortran_obj)(const double *x, const int *n);

static const gm_fortran_obj GM_BUILTINS[] = {
    furasn_, fush5_, fush7_, fush10_, fuhar3_, fuhar6_, fubran_, fugold_,
};
#define GM_N_BUILTINS \
    ((int)(sizeof(GM_BUILTINS) / sizeof(GM_BUILTINS[0])))

static gm_objective_fn gm_obj = NULL;
static void *gm_obj_data = NULL;
static int gm_builtin = -1; /* >=0 selects GM_BUILTINS[gm_builtin] */

static gm_constraint_fn gm_con = NULL;
static void *gm_con_data = NULL;

static long gm_evals = 0;

/* Optional evaluation trace */
static double *gm_trace_x = NULL;
static double *gm_trace_f = NULL;
static long gm_trace_cap = 0; /* number of evaluations reservable */
static long gm_trace_len = 0;
static int gm_trace_dim = 0;

void gm_set_objective(gm_objective_fn f, void *data) {
    gm_obj = f;
    gm_obj_data = data;
    gm_builtin = -1;
}

int gm_set_builtin(int idx) {
    if (idx < 0 || idx >= GM_N_BUILTINS) return -1;
    gm_builtin = idx;
    gm_obj = NULL;
    gm_obj_data = NULL;
    return 0;
}

int gm_n_builtins(void) { return GM_N_BUILTINS; }

void gm_set_constraints(gm_constraint_fn f, void *data) {
    gm_con = f;
    gm_con_data = data;
}

void gm_reset_evals(void) { gm_evals = 0; }
long gm_get_evals(void) { return gm_evals; }

/* Reserve a trace of up to cap evaluations of dimension dim.
 * Returns 0 on success, -1 on allocation failure. */
int gm_trace_start(long cap, int dim) {
    free(gm_trace_x);
    free(gm_trace_f);
    gm_trace_x = NULL;
    gm_trace_f = NULL;
    gm_trace_len = 0;
    gm_trace_cap = 0;
    gm_trace_dim = 0;
    if (cap <= 0 || dim <= 0) return 0;
    gm_trace_x = (double *)malloc((size_t)cap * (size_t)dim * sizeof(double));
    gm_trace_f = (double *)malloc((size_t)cap * sizeof(double));
    if (gm_trace_x == NULL || gm_trace_f == NULL) {
        free(gm_trace_x);
        free(gm_trace_f);
        gm_trace_x = NULL;
        gm_trace_f = NULL;
        return -1;
    }
    gm_trace_cap = cap;
    gm_trace_dim = dim;
    return 0;
}

long gm_trace_length(void) { return gm_trace_len; }
int gm_trace_dimension(void) { return gm_trace_dim; }
const double *gm_trace_points(void) { return gm_trace_x; }
const double *gm_trace_values(void) { return gm_trace_f; }

void gm_trace_stop(void) {
    free(gm_trace_x);
    free(gm_trace_f);
    gm_trace_x = NULL;
    gm_trace_f = NULL;
    gm_trace_cap = 0;
    gm_trace_len = 0;
    gm_trace_dim = 0;
}

void gm_ats_set(const double *state15) { gmatse_(state15); }
void gm_ats_get(double *state15) { gmatge_(state15); }

/* Canonical fresh-process ATS seeds (BLOCK DATA in gm_util.f). */
static const double GM_ATS_SEED[15] = {
    0.86515, 0.90795, 0.66155, 0.66434, 0.56558, 0.12332, 0.69186, 0.03393,
    0.42502, 0.99224, 0.88955, 0.53758, 0.41686, 0.42163, 0.85181,
};

void gm_ats_reset(void) { gmatse_(GM_ATS_SEED); }

/* Called from Fortran: FI(X,N) -> GMCALL(X,N) */
double gmcall_(const double *x, const int *n) {
    double f;
    if (gm_builtin >= 0) {
        f = GM_BUILTINS[gm_builtin](x, n);
    } else if (gm_obj != NULL) {
        f = gm_obj(x, *n, gm_obj_data);
    } else {
        f = 0.0;
    }
    gm_evals++;
    if (gm_trace_len < gm_trace_cap && *n == gm_trace_dim) {
        memcpy(gm_trace_x + (size_t)gm_trace_len * (size_t)gm_trace_dim, x,
               (size_t)gm_trace_dim * sizeof(double));
        gm_trace_f[gm_trace_len] = f;
        gm_trace_len++;
    }
    return f;
}

/* Called from Fortran: CONSTR(X,NX,R,K8) -> GMCON */
void gmcon_(const double *x, const int *nx, double *r, const int *k8) {
    if (gm_con != NULL) {
        gm_con(x, *nx, r, *k8, gm_con_data);
    } else {
        int i;
        for (i = 0; i < *k8; i++) r[i] = 0.0;
    }
}
