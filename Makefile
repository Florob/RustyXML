SRC := base.rs Parser.rs ElementBuilder.rs
RUSTCFLAGS := -O -Z debug-info

all: demo libxml.dummy doc


libxml.dummy: xml.rc ${SRC}
	rustc $< -o $@ ${RUSTCFLAGS}
	touch $@

demo: demo.rs libxml.dummy
	rustc $< -o $@ -L . ${RUSTCFLAGS}

xmltest: xml.rc ${SRC}
	rustc $< -o $@ -L . ${RUSTCFLAGS} --test

test: xmltest
	./xmltest

bench: xmltest
	./xmltest --bench

doc: doc/xml.md

doc/xml.md: xml.rc ${SRC}
	rustdoc --output-format markdown --output-dir doc --output-style doc-per-crate $<

clean:
	rm -f *.so *.dll *.dylib *.dummy demo xmltest doc/xml.md

.PHONY: clean test doc
