all: test lib

test: test.rs lib
	rustc test.rs -L . -O

lib: xml.rs
	rustc --lib xml.rs -O

.PHONY: lib
