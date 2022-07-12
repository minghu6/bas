.PHONY: libbas.a  # which enable rebuild always
.PHONY: vec.o
.PHONY: vec.so
.PHONY: aa.o
.PHONY: lib.o
.PHONY: cstr.o

libbas.a: vec.o aa.o cstr.o lib.o
	@ ar rcs $@ $^

vec.o: src/vec.c
	@ gcc -c -g -fpic -o $@ $^

vec.so.1.0: src/vec.c
	# The -Wl option passes options along to the linker (in this case the -soname linker option)
	@ gcc -shared -fpic -Wl,-soname,vec.so -o $@ $^

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

