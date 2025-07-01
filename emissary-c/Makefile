CARGO_TARGET_DIR = ../target
RUST_LIB_DIR = $(CARGO_TARGET_DIR)/release
RUST_LIB = emissary_c

CC = gcc
CFLAGS = -Wall -Wextra -std=c99
LDFLAGS = -L$(RUST_LIB_DIR) -l$(RUST_LIB) -lpthread -ldl -lz -lm
STATIC_LDFLAGS = $(LDFLAGS) -static

# Default target
all: example

# Build the Rust library first
rust-lib:
	cargo build --release

# Build the example (dynamic linking)
example: examples/simple_usage.c rust-lib
	$(CC) $(CFLAGS) $< $(LDFLAGS) -o $@

# Build the example with static linking
example-static: examples/simple_usage.c rust-lib
	$(CC) $(CFLAGS) $< $(STATIC_LDFLAGS) -o $@

# Clean build artifacts
clean:
	rm -f example example-static
	cargo clean

.PHONY: all rust-lib clean
