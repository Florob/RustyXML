all: test lib

test: test.rs lib
	rustc test.rs -L . -O -Z debug-info

lib: xml.rc
	rustc xml.rc -O -Z debug-info

.PHONY: lib
