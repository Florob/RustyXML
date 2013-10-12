SRC := base.rs Parser.rs ElementBuilder.rs
RUSTCFLAGS := -O -Z debug-info
RUSTC ?= rustc
RUSTDOC ?= rustdoc

all: demo libxml.dummy doc


libxml.dummy: xml.rs ${SRC}
	${RUSTC} $< ${RUSTCFLAGS}
	touch $@

demo: demo.rs libxml.dummy
	${RUSTC} $< -o $@ -L . ${RUSTCFLAGS}

xmltest: xml.rs ${SRC}
	${RUSTC} $< -o $@ -L . ${RUSTCFLAGS} --test

test: xmltest
	./xmltest

bench: xmltest
	./xmltest --bench

doc: xml.rs ${SRC}
	${RUSTDOC} -w html -o doc $<

clean:
	rm -f *.so *.dll *.dylib *.dummy demo xmltest

.PHONY: clean test doc
