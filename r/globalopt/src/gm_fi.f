C Bridge trampolines for the GlobalMinimum library.
C
C The upstream routines resolve the objective FI(X,N) and the
C constraint routine CONSTR(X,NX,R,K8) at link time.  These
C trampolines forward both to C entry points (gmcall/gmcon) so a host
C language (R, Python, Rust) can register interpreter callbacks or
C select a compiled built-in objective without relinking.
C
      FUNCTION FI(X,N)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      FI=GMCALL(X,N)
      RETURN
      END
      SUBROUTINE CONSTR(X,NX,R,K8)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(NX),R(K8)
      CALL GMCON(X,NX,R,K8)
      RETURN
      END
C
C ATS generator state access: the additive lagged-Fibonacci generator
C keeps its 15-word state in COMMON /ATT/ (initialised by BLOCK DATA in
C gm_util.f).  GMATSE/GMATGE let wrappers reset the state to reproduce
C fresh-process runs or to reseed for replicated benchmark runs.
C
      SUBROUTINE GMATSE(S)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION S(15)
      COMMON /ATT/X(15)
      DO 1 I=1,15
    1 X(I)=S(I)
      RETURN
      END
      SUBROUTINE GMATGE(S)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION S(15)
      COMMON /ATT/X(15)
      DO 1 I=1,15
    1 S(I)=X(I)
      RETURN
      END
