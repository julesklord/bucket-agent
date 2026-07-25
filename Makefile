# Makefile for Bucket TUI AI coding agent
# Matches conventions for easy packaging and manual installation.

# Configurable variables
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
INSTALL ?= install

# Targets
.PHONY: all build release install uninstall check test fmt clippy clean help

all: release

help:
	@echo "Bucket Makefile Targets:"
	@echo "  make build       - Build the debug binary (target/debug/bucket)"
	@echo "  make release     - Build the release binary (target/release/bucket) [Default]"
	@echo "  make install     - Install the release binary to $(DESTDIR)$(BINDIR)"
	@echo "  make uninstall   - Remove the binary from $(DESTDIR)$(BINDIR)"
	@echo "  make check       - Run fast validation check (cargo check)"
	@echo "  make test        - Run test suite (cargo test)"
	@echo "  make fmt         - Format all codebase files (cargo fmt)"
	@echo "  make clippy      - Lint the codebase (cargo clippy)"
	@echo "  make clean       - Remove target directories and build artifacts"

# Prerequisite validation
verify-prereqs:
	@which $(CARGO) >/dev/null 2>&1 || (echo "Error: cargo is not installed. Install Rust first." && exit 1)
	@which protoc >/dev/null 2>&1 || which dotslash >/dev/null 2>&1 || \
		(echo "Warning: Neither 'protoc' nor 'dotslash' was found. Code generation may fail." && \
		 echo "Please install protobuf compiler (e.g. 'sudo apt install protobuf-compiler' or 'brew install protobuf').")

build: verify-prereqs
	$(CARGO) build -p bucket-bin

release: verify-prereqs
	$(CARGO) build -p bucket-bin --release

install: release
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 target/release/bucket $(DESTDIR)$(BINDIR)/bucket

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/bucket

check:
	$(CARGO) check -p bucket-bin

test: verify-prereqs
	$(CARGO) test --workspace

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace

clean:
	$(CARGO) clean
