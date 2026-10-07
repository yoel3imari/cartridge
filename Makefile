PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

CARGO ?= cargo
TARGET = target/release/cart
TARGET_ALIAS = target/release/cartridge

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
	$(TARGET) completions bash > completions/cart.bash
	$(TARGET) completions zsh > completions/_cart
	$(TARGET) completions fish > completions/cart.fish
	$(TARGET_ALIAS) completions bash > completions/cartridge.bash
	$(TARGET_ALIAS) completions zsh > completions/_cartridge
	$(TARGET_ALIAS) completions fish > completions/cartridge.fish

install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 $(TARGET) $(DESTDIR)$(BINDIR)/cart
	install -m 755 $(TARGET_ALIAS) $(DESTDIR)$(BINDIR)/cartridge
	# Shell completions
	install -d $(DESTDIR)$(DATADIR)/bash-completion/completions
	$(TARGET) completions bash > $(DESTDIR)$(DATADIR)/bash-completion/completions/cart
	$(TARGET_ALIAS) completions bash > $(DESTDIR)$(DATADIR)/bash-completion/completions/cartridge
	install -d $(DESTDIR)$(DATADIR)/zsh/site-functions
	$(TARGET) completions zsh > $(DESTDIR)$(DATADIR)/zsh/site-functions/_cart
	$(TARGET_ALIAS) completions zsh > $(DESTDIR)$(DATADIR)/zsh/site-functions/_cartridge
	install -d $(DESTDIR)$(DATADIR)/fish/vendor_completions.d
	$(TARGET) completions fish > $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/cart.fish
	$(TARGET_ALIAS) completions fish > $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/cartridge.fish

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/cart
	rm -f $(DESTDIR)$(BINDIR)/cartridge
	rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/cart
	rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/cartridge
	rm -f $(DESTDIR)$(DATADIR)/zsh/site-functions/_cart
	rm -f $(DESTDIR)$(DATADIR)/zsh/site-functions/_cartridge
	rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/cart.fish
	rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/cartridge.fish

clean:
	$(CARGO) clean
	rm -rf completions
