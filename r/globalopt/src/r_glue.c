/* .Call glue between R and the GlobalMinimum Fortran library.
 *
 * Objectives are passed either as an R closure (evaluated through a C
 * callback registered with the shim) or as an integer index selecting a
 * compiled built-in objective (no interpreter involvement per
 * evaluation).  All optimizer entry points validate their inputs on the
 * R side (see R/api.R) so the upstream INWR* error branches, which WRITE
 * to stdout, are never reached.
 *
 * Interface details for every routine (argument roles, IPAR/PAR layout,
 * limits, COMMON blocks) are documented in docs/FORTRAN_INTERFACES.md.
 */

#include <R.h>
#include <Rinternals.h>
#include <string.h>

#include "gm.h"

#define GM_BIG 1e300

/* COMMON blocks (gfortran mangling: lowercase + trailing underscore) */
extern struct {
    double fm;
    double xm[100];
} laik_; /* LBAYES best point */
extern double bs1_[1000];  /* evaluation values channel (BAYES1/UNT/LPMIN/ANAL*) */
extern int statis_[4];     /* first word = IFAIL */
extern double hrez1_[30];  /* ANAL1 influence measures */
extern struct {
    int iii;
    int mb1[60]; /* MB1(2,30), column-major */
} hrez_;
extern double ana1_[400]; /* ANAL2 R(20,20) */
extern double ana2_[400]; /* ANAL2 VK(20,20) eigenvectors */
extern double ana3_[20];  /* ANAL2 DX influence, initial system */
extern double ana4_[20];  /* ANAL2 DY influence, eigen system */

/* ---------------- R objective callback ---------------- */

static SEXP gm_r_fn = NULL;
static SEXP gm_r_env = NULL;
static int gm_r_err = 0;

static double gm_r_objective(const double *x, int n, void *data) {
    SEXP xs, call, val;
    int err = 0;
    double f;

    (void)data;
    if (gm_r_err) return GM_BIG;

    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memcpy(REAL(xs), x, (size_t)n * sizeof(double));
    call = PROTECT(Rf_lang2(gm_r_fn, xs));
    val = R_tryEvalSilent(call, gm_r_env, &err);
    if (err || val == NULL) {
        gm_r_err = 1;
        UNPROTECT(2);
        return GM_BIG;
    }
    f = Rf_asReal(val);
    UNPROTECT(2);
    if (ISNAN(f)) return GM_BIG;
    return f;
}

/* ---------------- constraint callback (FLEXI/REQP) ---------------- */

static SEXP gm_r_confn = NULL;

static void gm_r_constraints(const double *x, int nx, double *r, int k8,
                             void *data) {
    SEXP xs, call, val;
    int err = 0, i, len;

    (void)data;
    for (i = 0; i < k8; i++) r[i] = 0.0;
    if (gm_r_err || gm_r_confn == NULL) return;

    xs = PROTECT(Rf_allocVector(REALSXP, nx));
    memcpy(REAL(xs), x, (size_t)nx * sizeof(double));
    call = PROTECT(Rf_lang2(gm_r_confn, xs));
    val = R_tryEvalSilent(call, gm_r_env, &err);
    if (err || val == NULL || !Rf_isReal(val)) {
        gm_r_err = 1;
        UNPROTECT(2);
        return;
    }
    len = (int)Rf_xlength(val);
    for (i = 0; i < k8 && i < len; i++) r[i] = REAL(val)[i];
    UNPROTECT(2);
}

/* ---------------- shared setup / teardown ---------------- */

static void gm_setup(SEXP objective, SEXP rho, int n, long trace_cap,
                     SEXP ats_state) {
    gm_r_err = 0;
    gm_r_fn = NULL;
    gm_r_confn = NULL;
    gm_r_env = R_GlobalEnv;

    if (Rf_isFunction(objective)) {
        gm_r_fn = objective;
        gm_r_env = rho;
        gm_set_objective(gm_r_objective, NULL);
    } else {
        if (gm_set_builtin(Rf_asInteger(objective)) != 0)
            Rf_error("invalid built-in objective index");
    }
    gm_set_constraints(NULL, NULL);

    if (Rf_isNull(ats_state)) {
        gm_ats_reset();
    } else {
        if (Rf_xlength(ats_state) != 15)
            Rf_error("ats_state must have length 15");
        gm_ats_set(REAL(ats_state));
    }

    gm_reset_evals();
    if (trace_cap > 0) {
        if (gm_trace_start(trace_cap, n) != 0)
            Rf_error("cannot allocate evaluation trace");
    } else {
        gm_trace_stop();
    }
}

static SEXP gm_result(int n, const double *best_x, double best_f) {
    long len = gm_trace_length();
    int dim = gm_trace_dimension();
    SEXP ans, names, par, points, values;
    int nprot = 0;

    if (gm_r_err) {
        gm_trace_stop();
        Rf_error("objective or constraint function signalled an error");
    }

    ans = PROTECT(Rf_allocVector(VECSXP, 6));
    nprot++;
    names = PROTECT(Rf_allocVector(STRSXP, 6));
    nprot++;
    SET_STRING_ELT(names, 0, Rf_mkChar("best_x"));
    SET_STRING_ELT(names, 1, Rf_mkChar("best_f"));
    SET_STRING_ELT(names, 2, Rf_mkChar("evals"));
    SET_STRING_ELT(names, 3, Rf_mkChar("points"));
    SET_STRING_ELT(names, 4, Rf_mkChar("values"));
    SET_STRING_ELT(names, 5, Rf_mkChar("status"));
    Rf_setAttrib(ans, R_NamesSymbol, names);

    par = PROTECT(Rf_allocVector(REALSXP, n));
    nprot++;
    memcpy(REAL(par), best_x, (size_t)n * sizeof(double));
    SET_VECTOR_ELT(ans, 0, par);
    SET_VECTOR_ELT(ans, 1, Rf_ScalarReal(best_f));
    SET_VECTOR_ELT(ans, 2, Rf_ScalarReal((double)gm_get_evals()));
    SET_VECTOR_ELT(ans, 5, Rf_ScalarInteger(statis_[0]));

    if (len > 0 && dim > 0) {
        const double *tx = gm_trace_points();
        const double *tf = gm_trace_values();
        long i;
        int j;
        points = PROTECT(Rf_allocMatrix(REALSXP, (int)len, dim));
        nprot++;
        values = PROTECT(Rf_allocVector(REALSXP, len));
        nprot++;
        for (i = 0; i < len; i++) {
            for (j = 0; j < dim; j++)
                REAL(points)[i + (long)j * len] = tx[i * dim + j];
            REAL(values)[i] = tf[i];
        }
        SET_VECTOR_ELT(ans, 3, points);
        SET_VECTOR_ELT(ans, 4, values);
    } else {
        SET_VECTOR_ELT(ans, 3, R_NilValue);
        SET_VECTOR_ELT(ans, 4, R_NilValue);
    }

    gm_trace_stop();
    UNPROTECT(nprot);
    return ans;
}

/* ---------------- optimizers ---------------- */

SEXP C_gm_mig1(SEXP a, SEXP b, SEXP evals, SEXP objective, SEXP rho,
               SEXP trace, SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(evals);
    int ipar[30] = {0};
    int ipa = 0;
    double fm = 0.0;
    SEXP xs;

    ipar[0] = -1; /* IPR: no printing */
    ipar[1] = m;

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m : 0, ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    mig1_(REAL(xs), REAL(a), REAL(b), &n, &fm, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(1);
    return ans;
}

SEXP C_gm_mig2(SEXP a, SEXP b, SEXP evals, SEXP objective, SEXP rho,
               SEXP trace, SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(evals);
    int nm = n * m;
    int ipar[30] = {0};
    int ipa = 0;
    double fm = 0.0;
    SEXP xs, xn;

    ipar[0] = -1;
    ipar[1] = m;

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m : 0, ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    xn = PROTECT(Rf_allocVector(REALSXP, nm));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    mig2_(REAL(xs), REAL(a), REAL(b), &n, REAL(xn), &nm, &fm, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(2);
    return ans;
}

SEXP C_gm_bayes1(SEXP a, SEXP b, SEXP evals, SEXP initial_points,
                 SEXP objective, SEXP rho, SEXP trace, SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(evals);
    int lt = Rf_asInteger(initial_points);
    int nm = n * m;
    int ipar[30] = {0};
    int ipa = 0;
    double fm = 0.0;
    SEXP xs, xn;

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = lt;

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m : 0, ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    xn = PROTECT(Rf_allocVector(REALSXP, nm));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    bayes1_(REAL(xs), REAL(a), REAL(b), &n, REAL(xn), &nm, &fm, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(2);
    return ans;
}

SEXP C_gm_lbayes(SEXP x0, SEXP a, SEXP b, SEXP iterations, SEXP aniu,
                 SEXP beta, SEXP objective, SEXP rho, SEXP trace,
                 SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int it = Rf_asInteger(iterations);
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double f1 = 0.0;
    SEXP xs;

    ipar[0] = -1;
    ipar[1] = it;
    ipar[2] = 0; /* NIPA: no integer variables */
    par[0] = Rf_asReal(aniu);
    par[1] = Rf_asReal(beta);

    /* LBAYES performs a handful of FI evaluations per iteration */
    gm_setup(objective, rho, n, Rf_asLogical(trace) ? (20L * it + 50L) : 0,
             ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memcpy(REAL(xs), REAL(x0), (size_t)n * sizeof(double));
    lbayes_(REAL(xs), REAL(a), REAL(b), &n, &f1, ipar, par, &ipa, &ipaa);
    /* best point is kept in COMMON /LAIK/ */
    SEXP ans = gm_result(n, laik_.xm, laik_.fm);
    UNPROTECT(1);
    return ans;
}

SEXP C_gm_unt(SEXP a, SEXP b, SEXP evals, SEXP initial_points,
              SEXP max_minima, SEXP objective, SEXP rho, SEXP trace,
              SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(evals);
    int nm = n * m;
    int ipar[30] = {0};
    int ipa = 0;
    double fm = 0.0;
    SEXP xs, xn;

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = Rf_asInteger(initial_points); /* 0 = auto (written back) */
    ipar[3] = Rf_asInteger(max_minima);

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m : 0, ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    xn = PROTECT(Rf_allocVector(REALSXP, nm));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    unt_(REAL(xs), REAL(a), REAL(b), &n, REAL(xn), &nm, &fm, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(2);
    return ans;
}

SEXP C_gm_glopt(SEXP a, SEXP b, SEXP evals, SEXP start_points,
                SEXP objective, SEXP rho, SEXP trace, SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int ipar[30] = {0};
    int ipa = 0;
    double fm = 0.0;
    SEXP xs;

    ipar[0] = -1;
    ipar[1] = Rf_asInteger(evals);
    ipar[2] = Rf_asInteger(start_points);

    gm_setup(objective, rho, n,
             Rf_asLogical(trace) ? (long)Rf_asInteger(evals) + 100L : 0,
             ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    glopt_(REAL(xs), REAL(a), REAL(b), &n, &fm, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(1);
    return ans;
}

SEXP C_gm_lpmin(SEXP a, SEXP b, SEXP analysis_evals, SEXP search_evals,
                SEXP objective, SEXP rho, SEXP trace, SEXP ats_state) {
    int n = (int)Rf_xlength(a);
    int m1 = Rf_asInteger(analysis_evals);
    int ml = Rf_asInteger(search_evals);
    int nm = n * (m1 > 0 ? m1 : 1);
    int ipar[30] = {0};
    int ipa = 0;
    double fmin = 0.0;
    SEXP xs, x2;

    ipar[0] = -1;
    ipar[1] = m1;
    ipar[2] = ml;

    gm_setup(objective, rho, n,
             Rf_asLogical(trace) ? (long)(m1 > 0 ? m1 : 0) + ml : 0,
             ats_state);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    x2 = PROTECT(Rf_allocVector(REALSXP, nm));
    memset(REAL(xs), 0, (size_t)n * sizeof(double));
    lpmin_(REAL(xs), REAL(a), REAL(b), &n, REAL(x2), &nm, &fmin, ipar, &ipa);
    SEXP ans = gm_result(n, REAL(xs), fmin);
    UNPROTECT(2);
    return ans;
}

SEXP C_gm_extr(SEXP bp, SEXP ep, SEXP evals, SEXP model_evals, SEXP acc_y,
               SEXP acc_x, SEXP objective, SEXP rho, SEXP trace) {
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double xm = 0.0, ym = 0.0;
    double bpv = Rf_asReal(bp), epv = Rf_asReal(ep);
    int m = Rf_asInteger(evals);

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = Rf_asInteger(model_evals);
    par[0] = Rf_asReal(acc_y);
    par[1] = Rf_asReal(acc_x);

    gm_setup(objective, rho, 1, Rf_asLogical(trace) ? m + 10L : 0,
             R_NilValue);
    extr_(&xm, &bpv, &epv, &ym, ipar, par, &ipa, &ipaa);
    return gm_result(1, &xm, ym);
}

SEXP C_gm_exkor(SEXP x0, SEXP a, SEXP b, SEXP evals, SEXP model_evals,
                SEXP cycles, SEXP first_coord, SEXP acc, SEXP objective,
                SEXP rho, SEXP trace) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(evals);
    int kc = Rf_asInteger(cycles);
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double fm = 0.0;
    double accv = Rf_asReal(acc);
    int i;
    SEXP xs;

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = Rf_asInteger(model_evals);
    ipar[3] = kc;
    ipar[4] = Rf_asInteger(first_coord);
    par[0] = accv;
    for (i = 0; i < n; i++) par[i + 1] = accv;

    gm_setup(objective, rho, n,
             Rf_asLogical(trace) ? (long)m * n * kc + 100L : 0, R_NilValue);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memcpy(REAL(xs), REAL(x0), (size_t)n * sizeof(double));
    exkor_(REAL(xs), REAL(a), REAL(b), &n, &fm, ipar, par, &ipa, &ipaa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(1);
    return ans;
}

SEXP C_gm_mivar4(SEXP x0, SEXP a, SEXP b, SEXP max_evals, SEXP nstop,
                 SEXP imax, SEXP xeps, SEXP eps, SEXP eps1, SEXP delta,
                 SEXP objective, SEXP rho, SEXP trace) {
    int n = (int)Rf_xlength(a);
    int m = Rf_asInteger(max_evals);
    int nm = n * (n + 1) / 2;
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double fm = 0.0;
    SEXP xs, b1;

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = Rf_asInteger(nstop);
    ipar[3] = Rf_asInteger(imax);
    par[0] = Rf_asReal(xeps);
    par[1] = Rf_asReal(eps);
    par[2] = Rf_asReal(eps1);
    par[3] = Rf_asReal(delta);

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m + 100L : 0,
             R_NilValue);
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    b1 = PROTECT(Rf_allocVector(REALSXP, nm));
    memcpy(REAL(xs), REAL(x0), (size_t)n * sizeof(double));
    mivar4_(REAL(xs), REAL(a), REAL(b), &n, REAL(b1), &nm, &fm, ipar, par,
            &ipa, &ipaa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(2);
    return ans;
}

SEXP C_gm_flexi(SEXP x0, SEXP max_evals, SEXP n_eq, SEXP n_ineq, SEXP size,
                SEXP conver, SEXP objective, SEXP rho, SEXP constraints,
                SEXP trace) {
    int n = (int)Rf_xlength(x0);
    int m = Rf_asInteger(max_evals);
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double ff = 0.0;
    SEXP xs;

    ipar[0] = -1;
    ipar[1] = m;
    ipar[2] = Rf_asInteger(n_eq);
    ipar[3] = Rf_asInteger(n_ineq);
    par[0] = Rf_asReal(size);
    par[1] = Rf_asReal(conver);

    gm_setup(objective, rho, n, Rf_asLogical(trace) ? m + 200L : 0,
             R_NilValue);
    if (Rf_isFunction(constraints)) {
        gm_r_confn = constraints;
        gm_set_constraints(gm_r_constraints, NULL);
    }
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    memcpy(REAL(xs), REAL(x0), (size_t)n * sizeof(double));
    flexi_(REAL(xs), &n, &ff, ipar, par, &ipa, &ipaa);
    SEXP ans = gm_result(n, REAL(xs), ff);
    UNPROTECT(1);
    return ans;
}

SEXP C_gm_reqp(SEXP x0, SEXP imax, SEXP n_eq, SEXP n_ineq, SEXP r1,
               SEXP scale, SEXP delta, SEXP eps, SEXP objective, SEXP rho,
               SEXP constraints, SEXP trace) {
    int n = (int)Rf_xlength(x0);
    int ipar[30] = {0};
    int ipa = 0, ipaa = 0;
    double par[30] = {0};
    double fm = 0.0;
    SEXP xs, b1, q, ja;

    ipar[0] = -1;
    ipar[1] = Rf_asInteger(imax);
    ipar[2] = Rf_asInteger(n_eq);
    ipar[3] = Rf_asInteger(n_ineq);
    par[0] = Rf_asReal(r1);
    par[1] = Rf_asReal(scale);
    par[2] = Rf_asReal(delta);
    par[3] = Rf_asReal(eps);

    gm_setup(objective, rho, n,
             Rf_asLogical(trace) ? 100L * (Rf_asInteger(imax) + 2L) * n : 0,
             R_NilValue);
    if (Rf_isFunction(constraints)) {
        gm_r_confn = constraints;
        gm_set_constraints(gm_r_constraints, NULL);
    }
    xs = PROTECT(Rf_allocVector(REALSXP, n));
    b1 = PROTECT(Rf_allocVector(REALSXP, n * n));
    q = PROTECT(Rf_allocVector(REALSXP, n * n));
    ja = PROTECT(Rf_allocVector(REALSXP, 100 * n));
    memcpy(REAL(xs), REAL(x0), (size_t)n * sizeof(double));
    reqp_(REAL(xs), REAL(b1), REAL(q), REAL(ja), &n, &fm, ipar, par, &ipa,
          &ipaa);
    SEXP ans = gm_result(n, REAL(xs), fm);
    UNPROTECT(4);
    return ans;
}

/* ---------------- analysis routines ----------------
 * ANAL1/ANAL2 do not evaluate FI: they analyze a supplied design
 * (points matrix, R column-major, len x n) with matching objective
 * values, which we place in COMMON /BS1/. */

static void gm_pack_points(SEXP points, int len, int n, double *xx) {
    int i, j;
    for (i = 0; i < len; i++)
        for (j = 0; j < n; j++)
            xx[(long)i * n + j] = REAL(points)[i + (long)j * len];
}

SEXP C_gm_anal1(SEXP xp, SEXP xg, SEXP points, SEXP values, SEXP harmonics,
                SEXP max_selected, SEXP interactions) {
    int n = (int)Rf_xlength(xp);
    int len = (int)Rf_xlength(values);
    int nm = n * len;
    int ipar[30] = {0};
    int ipa = 0;
    int i;
    SEXP xx, xwork, ans, names, infl, vars;

    ipar[0] = -1;
    ipar[1] = len;
    ipar[2] = Rf_asInteger(harmonics);
    ipar[3] = Rf_asInteger(max_selected);
    ipar[4] = Rf_asInteger(interactions);

    xx = PROTECT(Rf_allocVector(REALSXP, nm));
    xwork = PROTECT(Rf_allocVector(REALSXP, nm));
    gm_pack_points(points, len, n, REAL(xx));
    for (i = 0; i < len && i < 1000; i++) bs1_[i] = REAL(values)[i];

    anal1_(REAL(xp), REAL(xg), &n, REAL(xx), REAL(xwork), &nm, ipar, &ipa);

    int kv = hrez_.iii;
    if (kv < 0) kv = 0;
    if (kv > 30) kv = 30;
    ans = PROTECT(Rf_allocVector(VECSXP, 3));
    names = PROTECT(Rf_allocVector(STRSXP, 3));
    SET_STRING_ELT(names, 0, Rf_mkChar("influence"));
    SET_STRING_ELT(names, 1, Rf_mkChar("variables"));
    SET_STRING_ELT(names, 2, Rf_mkChar("status"));
    Rf_setAttrib(ans, R_NamesSymbol, names);

    infl = PROTECT(Rf_allocVector(REALSXP, kv));
    memcpy(REAL(infl), hrez1_, (size_t)kv * sizeof(double));
    vars = PROTECT(Rf_allocMatrix(INTSXP, kv, 2));
    for (i = 0; i < kv; i++) {
        INTEGER(vars)[i] = hrez_.mb1[2 * i];
        INTEGER(vars)[i + kv] = hrez_.mb1[2 * i + 1];
    }
    SET_VECTOR_ELT(ans, 0, infl);
    SET_VECTOR_ELT(ans, 1, vars);
    SET_VECTOR_ELT(ans, 2, Rf_ScalarInteger(statis_[0]));
    UNPROTECT(6);
    return ans;
}

SEXP C_gm_anal2(SEXP a, SEXP b, SEXP points, SEXP values) {
    int n = (int)Rf_xlength(a);
    int len = (int)Rf_xlength(values);
    int nm = n * len;
    int ipar[30] = {0};
    int ipa = 0;
    int i, j;
    SEXP xx, xwork, ans, names, dx, dy, vk;

    ipar[0] = -1;
    ipar[1] = len;

    xx = PROTECT(Rf_allocVector(REALSXP, nm));
    xwork = PROTECT(Rf_allocVector(REALSXP, nm));
    gm_pack_points(points, len, n, REAL(xx));
    for (i = 0; i < len && i < 1000; i++) bs1_[i] = REAL(values)[i];

    anal2_(REAL(a), REAL(b), &n, REAL(xx), REAL(xwork), &nm, ipar, &ipa);

    ans = PROTECT(Rf_allocVector(VECSXP, 4));
    names = PROTECT(Rf_allocVector(STRSXP, 4));
    SET_STRING_ELT(names, 0, Rf_mkChar("influence"));
    SET_STRING_ELT(names, 1, Rf_mkChar("influence_eigen"));
    SET_STRING_ELT(names, 2, Rf_mkChar("eigenvectors"));
    SET_STRING_ELT(names, 3, Rf_mkChar("status"));
    Rf_setAttrib(ans, R_NamesSymbol, names);

    dx = PROTECT(Rf_allocVector(REALSXP, n));
    dy = PROTECT(Rf_allocVector(REALSXP, n));
    memcpy(REAL(dx), ana3_, (size_t)n * sizeof(double));
    memcpy(REAL(dy), ana4_, (size_t)n * sizeof(double));
    vk = PROTECT(Rf_allocMatrix(REALSXP, n, n));
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            REAL(vk)[i + j * n] = ana2_[i + j * 20];
    SET_VECTOR_ELT(ans, 0, dx);
    SET_VECTOR_ELT(ans, 1, dy);
    SET_VECTOR_ELT(ans, 2, vk);
    SET_VECTOR_ELT(ans, 3, Rf_ScalarInteger(statis_[0]));
    UNPROTECT(7);
    return ans;
}

/* ---------------- utilities ---------------- */

SEXP C_gm_lptau(SEXP c_, SEXP n_) {
    double c = Rf_asReal(c_);
    int n = Rf_asInteger(n_);
    SEXP x = PROTECT(Rf_allocVector(REALSXP, n));
    lptau_(&c, &n, REAL(x));
    UNPROTECT(1);
    return x;
}

SEXP C_gm_eval_builtin(SEXP idx, SEXP x) {
    int n = (int)Rf_xlength(x);
    int i = Rf_asInteger(idx);
    double f;
    if (gm_set_builtin(i) != 0) Rf_error("invalid built-in objective index");
    gm_reset_evals();
    {
        extern double gmcall_(const double *x, const int *n);
        f = gmcall_(REAL(x), &n);
    }
    return Rf_ScalarReal(f);
}

SEXP C_gm_ats_get(void) {
    SEXP s = PROTECT(Rf_allocVector(REALSXP, 15));
    gm_ats_get(REAL(s));
    UNPROTECT(1);
    return s;
}

SEXP C_gm_ats_set(SEXP s) {
    if (Rf_xlength(s) != 15) Rf_error("ats state must have length 15");
    gm_ats_set(REAL(s));
    return R_NilValue;
}
