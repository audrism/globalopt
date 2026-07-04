#include <stdio.h>
#include <values.h>
#include <float.h>
#include <math.h>

main ()
{
	printf ("i1mach(5)=%d\n", sizeof(int)*8);
	printf ("i1mach(6)=%d\n", sizeof(int));
	printf ("i1mach(7)=%d\n", 2);
	printf ("i1mach(8)=%d\n", 31);
	printf ("i1mach(9)=%d\n", MAXINT);


	printf ("i1mach(10)=%d\n", FLT_RADIX);
	printf ("i1mach(11)=%d\n", FLT_MANT_DIG);
	printf ("i1mach(12)=%d\n", FLT_MIN_EXP);
	printf ("i1mach(13)=%d\n", FLT_MAX_EXP);
	printf ("i1mach(14)=%d\n", DBL_MANT_DIG);
	printf ("i1mach(15)=%d\n", DBL_MIN_EXP);
	printf ("i1mach(16)=%d\n", DBL_MAX_EXP);

	printf ("r1mach()=%.20e\n", FLT_MIN);
	printf ("r1mach()=%.20e\n", FLT_MAX);
	printf ("r1mach()=%.20e\n", FLT_EPSILON);
	printf ("r1mach()=%.20e\n", FLT_EPSILON);
	printf ("r1mach()=%.20e\n", (float)log10(2.0));
	printf ("d1mach()=%.20le\n", DBL_MIN);
	printf ("d1mach()=%.20le\n", DBL_MAX);
	printf ("d1mach()=%.20le\n", DBL_EPSILON);
	printf ("d1mach()=%.20le\n", DBL_EPSILON);
	printf ("d1mach()=%.20le\n", (double)log10(2.0));
}	



	
