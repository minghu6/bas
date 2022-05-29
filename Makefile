BARE_COMPILER=basc

build:
	@ cargo build

copybin: build
	@ cp ./target/debug/${BARE_COMPILER} .

getbasc: build copybin

genlib:
	@ cargo build -p rsc --release
	@ cp ./target/release/librsc.so ./runtime/
	@ cargo build -p mixin --release
	@ cp ./target/release/libmixin.a ./lib/
	@ cp ./target/release/libmixin.so ./lib/

testexp0: getbasc
	@ ./${BARE_COMPILER} ./examples/exp0.bath

testlex:
	cargo test -- --nocapture test_lex

testparser:
	cargo test -- --nocapture test_parser

testanalyze:
	RUST_BACKTRACE=1 cargo test -- --nocapture test_analyze

testcodegen:
	cargo test -- --nocapture test_codegen

dump:
	@ objdump -xsd ./output.o


.PHONY: clean
clean:
	@ rm -f *.so *.o ${BARE_COMPILER} main *.out *.d *.ll
