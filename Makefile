BARE_COMPILER=basc

build:
	@ cargo build

copybin: build
	@ cp ./target/debug/${BARE_COMPILER} .

getbasc: build copybin

testexp0: getbasc
	@ RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp0.bath exp0 -O2
	@ ./exp0

testexp1: getbasc
	@ RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp1.bath exp1 -O2
	@ ./exp1

testcodegen:
	cargo test -- --nocapture test_codegen

testboot:
	RUST_BACKTRACE=1 cargo test -- --nocapture test_boot

getlib:
	@ cd clib && make libbas.a && mv libbas.a ../

dump:
	@ objdump -xsd ./output.o


.PHONY: clean
clean:
	@ rm -f *.so *.o ${BARE_COMPILER} main *.out *.d *.ll
