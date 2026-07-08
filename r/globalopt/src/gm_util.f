C MINIMUM Copyright (c) 1989, by Jonas Mockus
C You may give out copies of this software; for conditions see the
C file COPYING included with this distribution.
C
C Double-precision (REAL*8) conversion of the upstream i1mach1.f
C utility bundle for the real.8 build of the GlobalMinimum library.
C The upstream real.8/ tree hand-converts the algorithm files to
C IMPLICIT REAL*8 but ships the utilities (ATS, LPTAU, machine
C constants, test objectives) in default REAL; on the historical x87
C ABI a REAL function result could be read as REAL*8 by callers, but
C on x86-64 SSE it cannot, so these utilities are converted here
C explicitly.  Numeric literals are written as D0 constants, matching
C an f77 -r8 style promoted build.
C
      INTEGER FUNCTION I1MACH(I)
C     PORT FOR IBM PC/AT
      INTEGER*4 IMACH(16)
      DATA IMACH/5,6,7,6,32,4,
     *2,31,2147483647,2,23,-128,127,52,-1024,1023/
      IF(I.LT.1.OR.I.GT.16) GO TO 1
      I1MACH=IMACH(I)
      RETURN
    1 I1MACH=6
      RETURN
      END
      FUNCTION R1MACH(I)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION RMACH(5)
      DATA RMACH/0.30D-38,0.17D+39,
     *0.119D-06,0.230D-06,0.301D+00/
      J=I
      IF (J.LT.1.OR.J.GT.5) J=5
      R1MACH=RMACH(J)
      RETURN
      END
      DOUBLE PRECISION FUNCTION D1MACH(I)
      DOUBLE PRECISION DMACH(5)
      INTEGER J
      DATA DMACH/0.419D-308,0.167D+309,2.22D-16,4.44D-16,3.01D-01/
      J=I
      IF (J.LT.1.OR.J.GT.5) J=5
      D1MACH=DMACH(J)
      RETURN
      END
      FUNCTION ATS(J)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      COMMON /ATT/X(15)
      X1=X(1)+X(15)
      IF (X1.GT.1.D0) X1=X1-1.D0
      DO 1 I=2,15
    1 X(I-1)=X(I)
      X(15)=X1
      ATS=X1
      RETURN
      END
      SUBROUTINE LPTAU(C,N,X)
C
C      PORTABLE GENERATOR OF LP-SEQUENCES
C      INPUT:
C      C    - NUMBER OF POINT
C      OUTPUT
C      X(N) - GENERATED POINT
C      N    - DIMENSION OF X
C
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      REAL*8 X(N),NR(400),A1(100),A2(100),A3(80),A4(60),A5(60)
      EQUIVALENCE(A1(1),NR(1)),(A2(1),NR(101)),(A3(1),NR(201)),
     *(A4(1),NR(281)),(A5(1),NR(341))
      DATA A1/20*1.,1.,3.,1.,3.,1.,3.,1.,3.,3.,1.,3.,1.,3.,1.,3.,1.,
     *1.,3.,1.,3.,1.,5.,7.,7.,5.,1.,3.,3.,7.,5.,5.,7.,7.,1.,3.,3.,7.,
     *5.,1.,1.,1.,15.,11.,5.,3.,1.,7.,9.,13.,11.,1.,3.,7.,9.,5.,13.,
     *13.,11.,3.,15.,1.,17.,13.,7.,15.,9.,31.,9.,3.,27.,15.,29.,21.,
     *23.,19.,11.,25.,7.,13.,17./
      DATA A2/1.,51.,61.,43.,51.,59.,47.,57.,35.,53.,19.,51.,61.,37.,
     *33.,7.,5.,11.,39.,63.,1.,85.,67.,49.,125.,25.,109.,43.,89.,69.,
     *113.,47.,55.,97.,3.,37.,83.,103.,27.,13.,1.,255.,79.,147.,141.,
     *89.,173.,43.,9.,25.,115.,97.,19.,97.,197.,101.,255.,29.,203.,
     *65.,1.,257.,465.,439.,177.,321.,181.,225.,235.,103.,411.,233.,
     *59.,353.,329.,463.,385.,111.,475.,451.,1.,771.,721.,1013.,759.,
     *835.,949.,113.,929.,615.,157.,39.,761.,169.,983.,657.,647.,581.,
     *505.,833./
      DATA A3/1.,1285.,823.,727.,267.,833.,471.,1601.,1341.,913.,1725.,
     *2021.,1905.,375.,893.,1599.,415.,605.,819.,975.,1.,3855.,4091.,
     *987.,1839.,4033.,2515.,579.,3863.,977.,3463.,2909.,3379.,1349.,
     *3739.,347.,387.,2881.,2821.,1873.,1.,4369.,4125.,5889.,6929.,3913
     *.,6211.,1731.,1347.,6197.,2817.,5459.,8119.,5121.,7669.,2481.,
     *7101.,2677.,1405.,7423.,1.,13107.,4141.,6915.,16241.,11643.,2147.
     *,11977.,4417.,14651.,9997.,2615.,13207.,13313.,2671.,5201.,11469.
     *,14855.,12165.,5837./
      DATA A4/1.,21845.,28723.,16647.,16565.,18777.,3169.,7241.,5087.,
     *2507.,7451.,13329.,8965.,19457.,18391.,3123.,11699.,721.,709.,
     *20481.,1.,65535.,45311.,49925.,17139.,35225.,35873.,63609.,
     *12631.,27109.,12055.,35887.,9997.,1033.,31161.,32253.,15865.,
     *26903.,41543.,12291.,1.,65537.,53505.,116487.,82207.,102401.,
     *33841.,81003.,103445.,5205.,44877.,97323.,75591.,62487.,12111.,
     *78043.,49173.,100419.,57545.,86017./
      DATA A5/1.,196611.,250113.,83243.,50979.,45059.,99889.,15595.,
     *152645.,91369.,24895.,83101.,226659.,250917.,259781.,63447.,
     *147489.,206167.,77163.,12303.,1.,327685.,276231.,116529.,252717.,
     *36865.,247315.,144417.,130127.,302231.,508255.,320901.,187499.,
     *234593.,36159.,508757.,81991.,241771.,357231.,299025.,1.,983055.,
     *326411.,715667.,851901.,299009.,1032727.,685617.,775365.,172023.,
     *574033.,810643.,628265.,308321.,232401.,974837.,802875.,987201.,
     *378135.,774207./
      D(X1)=X1-AINT(X1)
      M=1+INT(LOG(C)/0.693147D0)
      DO 1 J=1,N
      S=0.D0
      DO 2 K=1,M
      NS=0
      DO 3 L=K,M
      NN=(L-1)*20+J
      B=NR(NN)
    3 NS=NS+INT(2.D0*D(C/2.D0**L))*INT(2.D0*D(B/2.D0**(L+1-K)))
      RS=DBLE(NS)
    2 S=S+D(0.5D0*RS)/2.D0**(K-1)
    1 X(J)=S
      RETURN
      END
      BLOCK DATA
      REAL*8 X
      COMMON /ATT/X(15)
      DATA X/0.86515D0,0.90795D0,0.66155D0,0.66434D0,0.56558D0,
     C0.12332D0,0.69186D0,0.03393D0,0.42502D0,0.99224D0,0.88955D0,
     C0.53758D0,0.41686D0,0.42163D0,0.85181D0/
      END
      FUNCTION FURASN(X,N)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      NN=N
      F=0.D0
      DO 10 I=1,NN
      XI=X(I)
   10 F=F+XI*XI-COS(18.D0*XI)
      AN=NN
      FURASN=F*(2.D0/AN)
      RETURN
      END
      FUNCTION FUSH5(X,N)
C     SHEKEL'S FAMILY
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      DIMENSION A1(5,4),C1(5)
      DATA A1/4.,1.,8.,6.,3.,4.,1.,8.,6.,7.,4.,1.,8.,6.,3.,4.,1.,8.,6.,
     C7./
      DATA C1/.1D0,.2D0,.2D0,.4D0,.4D0/
      F=0.D0
      DO 1 I=1,5
      F1=C1(I)
      DO 2 J=1,N
      F2=X(J)-A1(I,J)
    2 F1=F1+F2*F2
    1 F=F-1.D0/F1
      FUSH5=F
      RETURN
      END
      FUNCTION FUSH7(X,N)
C     SHEKEL'S FAMILY
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      DIMENSION A1(7,4),C1(7)
      DATA A1/4.,1.,8.,6.,3.,2.,5.,4.,1.,8.,6.,7.,9.,5.,4.,1.,8.,6.,3.,
     C2.,3.,4.,1.,8.,6.,7.,9.,3./
      DATA C1/.1D0,.2D0,.2D0,.4D0,.4D0,.6D0,.3D0/
      F=0.D0
      DO 1 I=1,7
      F1=C1(I)
      DO 2 J=1,N
      F2=X(J)-A1(I,J)
    2 F1=F1+F2*F2
    1 F=F-1.D0/F1
      FUSH7=F
      RETURN
      END
      FUNCTION FUSH10(X,N)
C     SHEKEL'S FAMILY
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      DIMENSION A1(10,4),C1(10)
      DATA A1/4.,1.,8.,6.,3.,2.,5.,8.,6.,7.,4.,1.,8.,6.,7.,9.,5.,1.,2.,
     C3.6,4.,1.,8.,6.,3.,2.,3.,8.,6.,7.,4.,1.,8.,6.,7.,9.,3.,1.,2.,3.6/
      DATA C1/.1D0,.2D0,.2D0,.4D0,.4D0,.6D0,.3D0,.7D0,.5D0,.5D0/
      F=0.D0
      DO 1 I=1,10
      F1=C1(I)
      DO 2 J=1,N
      F2=X(J)-A1(I,J)
    2 F1=F1+F2*F2
    1 F=F-1.D0/F1
      FUSH10=F
      RETURN
      END
      FUNCTION FUHAR3(X,N)
C     HARTMAN'S FAMILY
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      DIMENSION ALFA(4,3),C1(4),P(4,3)
      DATA ALFA/3.,.1D0,3.,.1D0,4*10.,30.,35.,30.,35./
      DATA C1/1.,1.2D0,3.,3.2D0/
      DATA P/0.3689D0,0.4699D0,0.1091D0,0.03815D0,0.117D0,0.4387D0,
     C0.8732D0,0.5743D0,0.2673D0,0.7470D0,0.5547D0,0.8828D0/
      F=0.D0
      DO 1 I=1,4
      F1=0.D0
      DO 2 J=1,N
      F2=X(J)-P(I,J)
    2 F1=F1-ALFA(I,J)*F2*F2
    1 F=F-C1(I)*EXP(F1)
      FUHAR3=F
      RETURN
      END
      FUNCTION FUHAR6(X,N)
C     HARTMAN'S FAMILY
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      DIMENSION ALFA(4,6),C1(4),P(4,6)
      DATA ALFA/10.,.05D0,3.,17.,3.,10.,3.5D0,8.,17.,17.,1.7D0,.05D0,
     C3.5D0,.1D0,10.,10.,1.7D0,8.,17.,.1D0,8.,14.,8.,14./
      DATA C1/1.,1.2D0,3.,3.2D0/
      DATA P/0.1312D0,0.2329D0,0.2348D0,0.4047D0,0.1696D0,0.4135D0,
     C0.1451D0,0.8828D0,0.5569D0,0.8307D0,0.3522D0,0.8732D0,0.0124D0,
     C0.3736D0,0.2883D0,0.5743D0,0.8283D0,0.1004D0,0.3047D0,0.1091D0,
     C0.5886D0,0.9991D0,0.6650D0,0.0381D0/
      F=0.D0
      DO 1 I=1,4
      F1=0.D0
      DO 2 J=1,N
      F2=X(J)-P(I,J)
    2 F1=F1-ALFA(I,J)*F2*F2
    1 F=F-C1(I)*EXP(F1)
      FUHAR6=F
      RETURN
      END
      FUNCTION FUBRAN(X,N)
C     BRANIN FUNCTION
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      X1 = X(1)
      X2 = X(2)
      FUBRAN=(X2-0.1292D0*X1*X1+1.59155D0*X1-6.D0)**2
     C+9.60211D0*COS(X1)+10.D0
      RETURN
      END
      FUNCTION FUGOLD(X,N)
C     GOLDSTEIN AND PRICE FUNCTION
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      DIMENSION X(N)
      X1=X(1)
      X2=X(2)
      X3=X1*X1
      X4=X2*X2
      X5=X1*X2
      FUGOLD=(1.D0+(X1+X2+1.D0)**2*(19.D0-14.D0*X1+3.D0*X3-14.D0*X2
     C+6.D0*X5+3.D0*X4))*
     C(30.D0+(2.D0*X1-3.D0*X2)**2*(18.D0-32.D0*X1+12.D0*X3+48.D0*X2
     C-36.D0*X5+27.D0*X4))
      RETURN
      END
