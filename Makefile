SRC := base.rs Parser.rs ElementBuilder.rs
RUSTCFLAGS := -O -Z debug-info

all: demo libxml.dummy


libxml.dummy: xml.rc ${SRC}
	rustc $< -o $@ ${RUSTCFLAGS}
	touch $@

demo: demo.rs libxml.dummy
	rustc $< -o $@ -L . ${RUSTCFLAGS}

xmltest: xml.rc ${SRC}
	rustc $< -o $@ -L . ${RUSTCFLAGS} --test

test: xmltest
	./xmltest

clean:
	rm -f *.so *.dll *.dylib *.dummy demo xmltest

.PHONY: clean test
