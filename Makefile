RUSTC ?= rustc
RUSTCFLAGS := -O -g
RUSTDOC ?= rustdoc

all: build/xmldemo doc

lib: build
	${RUSTC} ${RUSTCFLAGS} --out-dir build/ src/xml/lib.rs

build/xmldemo: src/bin/xmldemo.rs lib
	${RUSTC} ${RUSTCFLAGS} -L build -o $@ $<

build/xmltest: src/xml/test.rs lib
	${RUSTC} ${RUSTCFLAGS} --test -L build -o $@ $<

test: build/xmltest
	build/xmltest

doc:
	${RUSTDOC} src/xml/lib.rs

clean:
	$(RM) -rf build doc

build:
	mkdir build

.PHONY: all lib doc test clean
