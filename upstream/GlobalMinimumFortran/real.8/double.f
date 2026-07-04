      program ex
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      dimension x(2),a(2),b(2),xn(200),hes(3),ipar(30),par(30),h(2,2),
     *q(2,2),gc(100,2)
      data n,nm,nh,ipa,ipaa/2,200,3,0,0/,a/-0.25,-0125/,b/0.5,0.625/
      data ipar/1,5,5,1,100,2,100,0,5,0,0,50,1,3,0,50,1,3,0,40,30,5,
     *0,1000,10,0,50,100,2*0/
      data par/100.,3*1.e-4,0.05,0.9,0.3,1.e-5,1.,0.25,2*1.e-4,18*0./
    2 format(3i5,4f7.2,i5)
      call bayes1(x,a,b,n,xn,nm,fm,ipar,ipa)
      ipa=3
      call mivar4(x,a,b,n,hes,nh,fm,ipar,par,ipa,ipaa)
      ipa=7
      ipaa=4
      call lbayes(x,a,b,n,f,ipar,par,ipa,ipaa)
      ipa=10
      ipaa=6
      call flexi(x,n,fm,ipar,par,ipa,ipaa)
      ipa=14
      ipaa=8
      call reqp(x,h,q,g,n,fm,ipar,par,ipa,ipaa)
      write(*,3) x(1),x(2)
    3 format(2f5.2)
      ipa=14
      ipaa=10
      call flexi(x,n,fm,ipar,par,ipa,ipaa)
      ipa=18
      ipaa=12
      call unt(x,a,b,n,xn,nm,fm,ipar,ipa)
      ipa=22
      call glopt(x,a,b,n,fm,ipar,ipa)
      ipa=25
      call lpmin(x,a,b,n,xn,nm,fm,ipar,ipa)
      stop
      end

      function fi(x,n)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      dimension x(n)
      fi=furasn(x,n)
      return
      end


      subroutine constr(x,n,g,m)
      IMPLICIT INTEGER*4 (I - N)
      IMPLICIT REAL*8    (A - H, O - Z)
      dimension x(n),g(m)
      g(1)=x(1)+1.
      g(2)=x(2)+1.
      g(3)=1.-x(1)
      g(4)=1.-x(2)
      return
      end

