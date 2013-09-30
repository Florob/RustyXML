SRC := base.rs Parser.rs ElementBuilder.rs
RUSTCFLAGS := -O -Z debug-info
RUSTC ?= rustc
RUSTDOC ?= rustdoc

all: demo libxml.dummy doc


libxml.dummy: xml.rc ${SRC}
	${RUSTC} $< ${RUSTCFLAGS}
	touch $@

demo: demo.rs libxml.dummy
	${RUSTC} $< -o $@ -L . ${RUSTCFLAGS}

xmltest: xml.rc ${SRC}
	${RUSTC} $< -o $@ -L . ${RUSTCFLAGS} --test

test: xmltest
	./xmltest

bench: xmltest
	./xmltest --bench

doc: doc/xml.md

doc/xml.md: xml.rc ${SRC}
	${RUSTDOC} html -o doc $<

clean:
	rm -f *.so *.dll *.dylib *.dummy demo xmltest

.PHONY: clean test doc
