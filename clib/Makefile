.PHONY: libbas.a  # which enable rebuild always
.PHONY: vec.o
.PHONY: aa.o
.PHONY: lib.o
.PHONY: cstr.o

libbas.a: vec.o aa.o cstr.o lib.o
	@ ar rcs $@ $^

vec.o: src/vec.c
	@ gcc -c -g -fpic -o $@ $^

aa.o: src/aa.c
	@ gcc -c -g -fpic -o $@ $^

cstr.o: src/cstr.c
	@ gcc -c -g -fpic -o $@ $^

lib.o: src/lib.c
	@ gcc -c -g -fpic -o $@ $^

clean:
	@ rm -f *.o *.a

expandvec:
	@ gcc -E src/vec.c

expandaa:
	@ gcc -E src/aa.c

