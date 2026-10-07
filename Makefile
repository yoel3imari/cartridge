PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

CARGO ?= cargo
TARGET = target/release/aim

.PHONY: all build release test lint fmt install uninstall clean completions

all: release

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test

lint:
	$(CARGO) clippy --all-targets -- -D warnings

fmt:
	$(CARGO) fmt --check

completions: release
	mkdir -p completions
	$(TARGET) completions bash > completions/aim.bash
	$(TARGET) completions zsh > completions/_aim
	$(TARGET) completions fish > completions/aim.fish

install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 $(TARGET) $(DESTDIR)$(BINDIR)/aim
	# Shell completions
	install -d $(DESTDIR)$(DATADIR)/bash-completion/completions
	$(TARGET) completions bash > $(DESTDIR)$(DATADIR)/bash-completion/completions/aim
	install -d $(DESTDIR)$(DATADIR)/zsh/site-functions
	$(TARGET) completions zsh > $(DESTDIR)$(DATADIR)/zsh/site-functions/_aim
	install -d $(DESTDIR)$(DATADIR)/fish/vendor_completions.d
	$(TARGET) completions fish > $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/aim.fish

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/aim
	rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/aim
	rm -f $(DESTDIR)$(DATADIR)/zsh/site-functions/_aim
	rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/aim.fish

clean:
	$(CARGO) clean
	rm -rf completions
