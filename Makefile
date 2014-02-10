RUSTC ?= rustc
RUSTCFLAGS := -O -g
RUSTDOC ?= rustdoc

all: build/xmldemo doc

lib: build
	${RUSTC} ${RUSTCFLAGS} --out-dir build/ src/xml/lib.rs

build/xmldemo: lib
	${RUSTC} ${RUSTCFLAGS} -L build -o $@ src/xmldemo/main.rs

build/xmltest: lib
	${RUSTC} ${RUSTCFLAGS} --test -L build -o $@ src/xml/test.rs

test: build/xmltest
	build/xmltest

doc:
	${RUSTDOC} src/xml/lib.rs

clean:
	$(RM) -rf build doc

build:
	mkdir build

.PHONY: all lib doc test clean
