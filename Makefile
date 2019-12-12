PREFIX = /usr/local
SUBQUOTE = subquote
OBJ = target/release/$(SUBQUOTE)
CARGO := $(shell command -v cargo 2> /dev/null)

build:
ifndef CARGO
	$(error "cargo is not available. Please install rustup (https://www.rust-lang.org/tools/install)")
endif
	$(CARGO) build --release

install:
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	cp -f $(OBJ) $(DESTDIR)$(PREFIX)/bin
	chmod 755 $(DESTDIR)$(PREFIX)/bin/$(SUBQUOTE)

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(SUBQUOTE)

.PHONY: install build uninstall