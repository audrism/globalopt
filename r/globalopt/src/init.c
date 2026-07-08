#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern SEXP C_gm_mig1(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_mig2(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_bayes1(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_lbayes(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP,
                        SEXP);
extern SEXP C_gm_unt(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_glopt(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_lpmin(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_extr(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_exkor(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP,
                       SEXP, SEXP);
extern SEXP C_gm_mivar4(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP,
                        SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_flexi(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP,
                       SEXP);
extern SEXP C_gm_reqp(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP,
                      SEXP, SEXP, SEXP);
extern SEXP C_gm_anal1(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_anal2(SEXP, SEXP, SEXP, SEXP);
extern SEXP C_gm_lptau(SEXP, SEXP);
extern SEXP C_gm_eval_builtin(SEXP, SEXP);
extern SEXP C_gm_ats_get(void);
extern SEXP C_gm_ats_set(SEXP);

static const R_CallMethodDef CallEntries[] = {
    {"C_gm_mig1", (DL_FUNC)&C_gm_mig1, 7},
    {"C_gm_mig2", (DL_FUNC)&C_gm_mig2, 7},
    {"C_gm_bayes1", (DL_FUNC)&C_gm_bayes1, 8},
    {"C_gm_lbayes", (DL_FUNC)&C_gm_lbayes, 10},
    {"C_gm_unt", (DL_FUNC)&C_gm_unt, 9},
    {"C_gm_glopt", (DL_FUNC)&C_gm_glopt, 8},
    {"C_gm_lpmin", (DL_FUNC)&C_gm_lpmin, 8},
    {"C_gm_extr", (DL_FUNC)&C_gm_extr, 9},
    {"C_gm_exkor", (DL_FUNC)&C_gm_exkor, 11},
    {"C_gm_mivar4", (DL_FUNC)&C_gm_mivar4, 13},
    {"C_gm_flexi", (DL_FUNC)&C_gm_flexi, 10},
    {"C_gm_reqp", (DL_FUNC)&C_gm_reqp, 12},
    {"C_gm_anal1", (DL_FUNC)&C_gm_anal1, 7},
    {"C_gm_anal2", (DL_FUNC)&C_gm_anal2, 4},
    {"C_gm_lptau", (DL_FUNC)&C_gm_lptau, 2},
    {"C_gm_eval_builtin", (DL_FUNC)&C_gm_eval_builtin, 2},
    {"C_gm_ats_get", (DL_FUNC)&C_gm_ats_get, 0},
    {"C_gm_ats_set", (DL_FUNC)&C_gm_ats_set, 1},
    {NULL, NULL, 0}};

void R_init_globalopt(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
