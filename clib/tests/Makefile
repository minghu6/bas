.PHONY: lib_test
.PHONY: regex_test

lib_test:
	@ gcc -g -DDEBUG= lib_test.c ../libbas.a -o lib_test
	@ ./lib_test

regex_test:
	@ gcc regex_test.c -o regex_test
	@ ./regex_test

clean:
	@rm -f *.o *.a
