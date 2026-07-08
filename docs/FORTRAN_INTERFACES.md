# GlobalMinimum Fortran routine interfaces (wrapper-ready spec)

Extracted from `upstream/GlobalMinimumFortran/real.8/` sources. All REAL
arguments are REAL*8; all `I..N` integers are INTEGER*4. IPAR/PAR are always
INTEGER(30)/REAL*8(30); IPA/IPAA are base offsets (use 0). `IPAR(IPA+1)` is
always the printing parameter IPR (wrappers pass -1 = silent). Objective is
the link-time function `FI(X,N)`; constraints (FLEXI, REQP) the link-time
`CONSTR(X,NX,R,K8)`. Validation failures set IFAIL=10 in COMMON /STATIS/
(first word) and return; wrappers must pre-validate so upstream WRITE error
paths are never reached.

Common state: COMMON /ATT/ (15-word ATS random state, used by MIG1, MIG2,
BAYES1(planner), UNT, GLOPT) persists across calls; wrappers reset it via
GMATSE before each run. /BS1/ (1000 doubles) holds evaluation values for
BAYES1/UNT/LPMIN and is the input channel for ANAL1/ANAL2.

| Routine | Signature | IPAR (after IPR) | PAR | Limits | Notes |
| --- | --- | --- | --- | --- | --- |
| MIG1 | MIG1(X,A,B,N,FM,IPAR,IPA) | M evals | - | N<=100 | X out |
| MIG2 | MIG2(X,A,B,N,XN,NM,FM,IPAR,IPA) | M | - | N<=20, M<=1000, NM>=N*M | X out |
| BAYES1 | BAYES1(X,A,B,N,XN,NM,FM,IPAR,IPA) | M, LT init pts | - | N<=20, M<=1000, 1<=LT<=M, NM>=N*M | X out |
| LBAYES | LBAYES(X,A,B,N,F1,IPAR,PAR,IPA,IPAA) | IT iters, NIPA int vars | ANIU, BETA | N<=100, IT>0, 0<=NIPA<=N | X in/out (in=start, out=LAST point); BEST point in COMMON /LAIK/FM,XM(100); F1 out = mean value; box handled by projection; deterministic |
| UNT | UNT(X,A,B,N,XN,NM,FM,IPAR,IPA) | M evals, LT init (0=auto, else 30<=LT<=M; written back!), ML max local minima | - | N<=20, M<=500, ML in 1..20, NM>=N*M | X out; IFAIL: 0 ML reached, 1 density, 2 M reached |
| EXTR | EXTR(XM,BP,EP,YM,IPAR,PAR,IPA,IPAA) | M evals, LT model evals | E1 acc(Y), E2 acc(X) | M<=500, 6<=LT<=M | 1-D; XM,YM out; FI called as FI(X,1) |
| EXKOR | EXKOR(X,A,B,N,FM,IPAR,PAR,IPA,IPAA) | M evals per 1-D search, LT (>=6), KC cycles, NO first coord | E1 acc(F), then N per-coord accs | N<=20, M<=500, 6<=LT<=M, 1<=NO<=N; PAR needs IPAA+N+1<=30 | X in/out |
| MIVAR4 | MIVAR4(X,A,B,NN,B1,NM,FM,IPAR,PAR,IPA,IPAA) | M evals, NSTOP, IMAX | XEPS, EPS, EPS1, DELT | NN<=100, NM>=NN*(NN+1)/2 | X in/out; B1 workspace; IFAIL 0..4 termination reason |
| FLEXI | FLEXI(Z,M1,FF,IPAR,PAR,IPA,IPAA) | NFMAX evals, NC eq, NIC ineq | SIZE simplex edge, CONVER | M1<=20, NC+NIC<=100 | Z in/out; NO bounds - region set by start+SIZE; CONSTR(X,M1,R,NC+NIC): R(1..NC) equalities (=0 feasible), R(NC+1..) inequalities (>=0 feasible) |
| GLOPT | GLOPT(XM,A,B,M,FM,IPAR,IPA) | FNMAX evals, PN start pts | - | M(dim)<=20, 1<=PN<=150 | XM out; M arg is DIMENSION; random via ATS |
| LPMIN | LPMIN(X,A,B,N,X2,NM,FMIN,IPAR,IPA) | M1 analysis evals (<0 none; 0 use order in IPAR(IPA+4..); else 10..300), ML search evals | - | N<=20, NM>=N*M1, /BS1/ cap 1000 | X out; deterministic (LPTAU) |
| ANAL1 | ANAL1(XP,XG,N,XX,X,NM,IPAR,IPA) | M pts (10..300), NH harmonics (1..7), NSF max selected (1..30), INP mode (1 or 2) | - | N<=20, NM>=N*M | does NOT call FI: preload values in /BS1/Y and points in XX; results in /HREZ1/DDD(30) (influence, sorted desc) and /HREZ/III,MB1(2,30) (variable indices; MB1(2,i)=0 for single var) |
| ANAL2 | ANAL2(A,B,N,XX,X,NM,IPAR,IPA) | M pts (10..300) | - | N<=20, NM>=N*M | does NOT call FI: preload /BS1/F; results in /ANA1/R(20,20), /ANA2/VK(20,20) eigvecs, /ANA3/DX(20) influence, /ANA4/DY(20) eigen influence |
| REQP | REQP(X,B1,Q,A,N,FM,IPAR,PAR,IPA,IPAA) | IMAX iters, NC eq, NIC ineq | R1 penalty, SCALE, DELTA fd-step, EPS | N<=100, NC+NIC<=100; B1(N,N), Q(N,N), A(100,N) workspaces | X in/out; CONSTR(X,N,G,NC+NIC) same sign convention as FLEXI; multipliers in /W7/Y(100) |

Realistic parameter templates (from upstream ex8.f/ex9.f):
- EXKOR: M=100, LT=6, KC=2, NO=1; PAR E1=0.01, per-coord acc 0.01
- MIVAR4: M=100, NSTOP=2, IMAX=100; XEPS=100., EPS=1e-4, EPS1=1e-4, DELT=1e-4
- LBAYES: IT=5, NIPA=0; ANIU=0.05, BETA=0.9
- FLEXI: NFMAX=50, NC=1, NIC=3; SIZE=0.3, CONVER=1e-5
- REQP: IMAX=50; R1=1., SCALE=0.25, DELTA=1e-4, EPS=1e-4
- UNT: M=40, LT=30, ML=5
- GLOPT: FNMAX=1000, PN=10
- LPMIN: M1=50, ML=100

Build requirements:
- Compile all Fortran with `-std=legacy -fdefault-real-8 -fdefault-double-8`
  (the real.8 tree assumes f77 -r8 style promotion; see gm_util.f header).
- Vendored-source patches (tools/sync_fortran.sh): mivar4 SRMIN/SRMAX ->
  SDMIN/SDMAX (restores unset variables); exkor COR/ALNORM -> CORXK/ALNMXK
  and anal2 FAKTKK -> FAKT22 (duplicate symbols vs extr.f/lpmin.f); glopt
  LOCOP DIMENSION scalars removed.
- COMMON blocks to access from C:
  /LAIK/ = struct { double fm; double xm[100]; }  (LBAYES result)
  /BS1/  = double y[1000]                          (values channel)
  /STATIS/ first int = IFAIL                       (termination code)
  /ATT/  = double state[15]                        (ATS random state)
  /HREZ1/, /HREZ/, /ANA1../ANA4/ as above (ANAL1/2 results)

Numerical smoke reference (this machine, builtin FURASN, a=(-0.25,-0.125),
b=(0.5,0.625)):
- LPTAU(1..4, n=2): (0.5,0.5), (0.25,0.75), (0.75,0.25), (0.125,0.625)
- BAYES1 M=200 LT=20: best_f=-1.9966149827762463, evals=200,
  first values 0.91709442215351888, -0.21484719829221782, -1.1207175656694048
- MIG2 M=200: best_f=-1.8296951132230181
