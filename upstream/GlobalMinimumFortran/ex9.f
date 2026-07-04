      program ex
      dimension x(2),a(2),b(2),ipar(30),par(30)
      data n,nm,ipa,ipaa/2,400,0,0/,a/-0.25,-0.125/,b/0.5,0.625/
      data ipar/0,100,6,2,1,25*0/
      data par/0.01,0.01,0.01,27*0./
      data x/2*0.1/
      call exkor(x,a,b,n,fm,ipar,par,ipa,ipaa)
      stop
      end
      function fi(x,n)
      dimension x(n)
      fi=furasn(x,n)
      return
      end
