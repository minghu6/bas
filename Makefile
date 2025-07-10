BARE_COMPILER=basc

build:
	@ cargo build

copybin: build
	@ cp ./target/debug/${BARE_COMPILER} .

getbasc: build copybin

runexp0:
	@ RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp0.bath -O2 exp0
	@ # RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp0.bath stderr -O2 -t lib -e llvm-ir
	@ ./exp0

testexp0: getbasc runexp0

testexp1: getbasc
	# @ RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp1.bath stderr -O2 -t lib -e llvm-ir
	@ RUST_BACKTRACE=1 ./${BARE_COMPILER} ./examples/exp1.bath exp1.o -O2 -t lib

testcodegen:
	cargo test -- --nocapture test_codegen

testboot:
	RUST_BACKTRACE=1 cargo test -- --nocapture test_boot

getlib:
	@ cd clib && make libbas.a && mv libbas.a ../

dump:
	@ objdump -xsd ./output.o

testspec:
	@ cargo test -- --nocapture test_spec_

.PHONY: clean
clean:
	@ rm -f *.so *.o ${BARE_COMPILER} main *.out *.d *.ll

clean-exp0:
	@ rm -f exp0

clean-lib:
	@ rm -f libbas.a
