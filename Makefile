all: test lib

test: test.rs lib
	rustc test.rs -L . -O -Z debug-info

lib: xml.rs
	rustc xml.rs -O -Z debug-info

.PHONY: lib
