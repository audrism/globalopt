      program ex
      dimension x(2),a(2),b(2),xn(400),xm(400),ipar(30)
      data n,nm,ipa/2,400,0/,a/-0.25,-0.125/,b/0.5,0.625/
      data ipar/-1,200,0,200,7,8,2,23*0/
c     write(*,2)
c   2 format(' n=?  ?  i3)
c     read(*,1) n
c   1 format(i3)
      call mig2(x,a,b,n,xm,nm,fm,ipar,ipa)
      ipa=2
      call anal1(a,b,n,xn,xm,nm,ipar,ipa)
      stop
      end
c     function fi2(x,n)
c     dimension x(n)
c     fi2=furasn(x,n)
c     return
c     end
      function fi(x,n)
      dimension x(n)
      fi=furasn(x,n)
      return
      end
