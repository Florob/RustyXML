RUSTC ?= rustc
RUSTCFLAGS := -O -Zdebug-info --out-dir build/
RUSTDOC ?= rustdoc

all: demo doc

lib: build
	${RUSTC} ${RUSTCFLAGS} src/xml/lib.rs

demo: lib
	${RUSTC} ${RUSTCFLAGS} -L build src/xmldemo/main.rs

test: build
	${RUSTC} ${RUSTCFLAGS} --test src/xml/lib.rs

doc:
	${RUSTDOC} src/xml/lib.rs

clean:
	$(RM) -rf build doc

build:
	mkdir build

.PHONY: all lib demo doc test clean 
