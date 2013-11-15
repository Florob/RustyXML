RUSTPKGFLAGS := -O -Zdebug-info
RUSTPKG ?= rustpkg
RUSTDOC ?= rustdoc

all: demo doc


lib:
	${RUSTPKG} build ${RUSTPKGFLAGS} xml

demo:
	${RUSTPKG} build ${RUSTPKGFLAGS} xmldemo

test:
	${RUSTPKG} test xml

doc:
	${RUSTDOC} src/xml/lib.rs

clean:
	${RUSTPKG} clean

.PHONY: all lib demo doc test clean 
