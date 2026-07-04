OBJECTS = anal1.o\
 anal2.o\
 bayes1.o\
 exkor.o\
 extr.o\
 flexi.o\
 glopt.o\
 lbayes.o\
 lpmin.o\
 mig1.o\
 mig2.o\
 mivar4.o\
 reqp.o\
 unt.o

.f.o: ; f77 -c $*.f


PC:   $(OBJECTS) i1mach1.o
	ar clr libgm.a $(OBJECTS) i1mach1.o
	rm $(OBJECTS) i1mach1.o

VAX:  $(OBJECTS) i1mach2.o
	ar clr libgm.a $(OBJECTS) i1mach2.o
	rm $(OBJECTS) i1mach2.o

mips: $(OBJECTS) i1mach3.o
	ar clr libgm.a $(OBJECTS) i1mach3.o
	ar ts libgm.a
	rm $(OBJECTS) i1mach3.o

cray: $(OBJECTS) i1mach4.o
	ar clr libgm.a $(OBJECTS) i1mach4.o
	rm $(OBJECTS) i1mach4.o

