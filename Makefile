RUSTC ?= rustc
RUSTCFLAGS := -O -g
RUSTDOC ?= rustdoc

all: build/xmldemo doc

lib: src/xml/lib.rs
	mkdir -p build
	${RUSTC} ${RUSTCFLAGS} --out-dir build/ $<

build/xmldemo: src/bin/xmldemo.rs lib
	mkdir -p build
	${RUSTC} ${RUSTCFLAGS} -L build -o $@ $<

build/xmltest: src/xml/lib.rs src/xml/Parser.rs src/xml/ElementBuilder.rs
	mkdir -p build
	${RUSTC} ${RUSTCFLAGS} --test -L build -o $@ $<

test: build/xmltest
	build/xmltest

doc:
	${RUSTDOC} src/xml/lib.rs

clean:
	$(RM) -rf build doc

.PHONY: all lib doc test clean
