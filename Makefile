BARE_COMPILER=basc

build:
	@ cargo build

copybin:
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

testllvm_iodemo:
	cargo test -- --nocapture test_io

testllvm_iterarray:
	cargo test -- --nocapture test_llvm_iterarray

testllvm_doseq:
	cargo test -- --nocapture test_llvm_doseq

testlex:
	cargo test -- --nocapture test_lex

testparser:
	cargo test -- --nocapture test_parser

testmls:
	cargo test -- --nocapture test_ml_simplifier

testcodegen:
	cargo test -- --nocapture test_codegen

dump:
	@ objdump -xsd ./output.o


.PHONY: clean
clean:
	@ rm -f *.so *.o ${BARE_COMPILER} main *.out *.d *.ll
