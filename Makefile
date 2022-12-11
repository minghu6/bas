BARE_COMPILER=basc

build:
	@ cargo build

copybin: build
	@ cp ./target/debug/${BARE_COMPILER} .

getbasc: build copybin

testexp0: getbasc
	@ ./${BARE_COMPILER} ./examples/exp0.bath exp0
	@ ./exp0

testlex:
	cargo test -- --nocapture test_lex

testparser:
	cargo test -- --nocapture test_parser

testanalyze:
	RUST_BACKTRACE=1 cargo test -- --nocapture test_analyze

testcodegen:
	cargo test -- --nocapture test_codegen

testcompile:
	cargo test -- --nocapture test_compile

testboot:
	RUST_BACKTRACE=1 cargo test -- --nocapture test_boot

getlib:
	@ cd clib && make libbas.a && mv libbas.a ../

dump:
	@ objdump -xsd ./output.o


.PHONY: clean
clean:
	@ rm -f *.so *.o ${BARE_COMPILER} main *.out *.d *.ll
