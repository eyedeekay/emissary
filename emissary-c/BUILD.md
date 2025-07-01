# Building Emissary C Bindings

This document provides detailed build instructions for different platforms.

## Prerequisites

- **Rust**: Version 1.70 or higher
- **Cargo**: Included with Rust
- **C Compiler**: 
    - Linux: GCC 7+ or Clang 6+
    - macOS: Xcode Command Line Tools (clang)
    - Windows: MSVC 2019+ or MinGW-w64

## Quick Build

```bash
# From the emissary project root
cargo build --package emissary-c --release
```

## Platform-Specific Instructions

### Linux (Ubuntu/Debian)

```bash
# Install dependencies
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Build the library
cargo build --package emissary-c --release

# Test the example (static linking)
cd emissary-c
gcc examples/simple_usage.c -L../target/release -lemissary_c -lpthread -ldl -lz -lm -static -o example
./example
```

### Linux (CentOS/RHEL/Fedora)

```bash
# Install dependencies
sudo dnf install gcc pkg-config openssl-devel

# Or for older versions:
# sudo yum install gcc pkg-config openssl-devel

# Build the library
cargo build --package emissary-c --release

# Test the example (static linking)
cd emissary-c
gcc examples/simple_usage.c -L../target/release -lemissary_c -lpthread -ldl -lz -lm -static -o example
./example
```

### macOS

```bash
# Install Xcode command line tools (if not already installed)
xcode-select --install

# Build the library
cargo build --package emissary-c --release

# Test the example (static linking)
cd emissary-c
clang examples/simple_usage.c -L../target/release -lemissary_c -lpthread -static -o example
./example
```

### Windows (MSVC)

```cmd
REM Ensure you have Rust and MSVC installed
REM Open "x64 Native Tools Command Prompt for VS"

REM Build the library
cargo build --package emissary-c --release

REM Test the example (static linking)
cd emissary-c
cl examples\simple_usage.c /I. /MT ..\target\release\emissary_c.lib ws2_32.lib userenv.lib /Fe:simple_usage.exe
simple_usage.exe
```

### Windows (MinGW)

```bash
# Using MSYS2/MinGW-w64
# Install dependencies: pacman -S mingw-w64-x86_64-gcc

# Build the library
cargo build --package emissary-c --release

# Test the example (static linking)
cd emissary-c
gcc examples/simple_usage.c -L../target/release -lemissary_c -lws2_32 -luserenv -static -o example.exe
./example.exe
```

## Library Outputs

After building, you'll find these files in `target/release/`:

### Linux
- `libemissary_c.a` - Static library (preferred)
- `libemissary_c.so` - Dynamic library

### macOS
- `libemissary_c.a` - Static library (preferred)
- `libemissary_c.dylib` - Dynamic library

### Windows
- `emissary_c.lib` - Static library (preferred)
- `emissary_c.dll` - Dynamic library
- `libemissary_c.a` - Static library (MinGW)

## Integration Examples

### CMake Integration

```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.10)
project(my_i2p_app)

# Find the emissary-c static library
find_library(EMISSARY_C_LIB 
        NAMES libemissary_c.a emissary_c.lib libemissary_c.a
        PATHS ./target/release
)

add_executable(my_app main.c)

# Static linking configuration
target_link_libraries(my_app ${EMISSARY_C_LIB})
set_target_properties(my_app PROPERTIES LINK_SEARCH_START_STATIC 1)
set_target_properties(my_app PROPERTIES LINK_SEARCH_END_STATIC 1)

# Platform-specific system libraries
if(UNIX AND NOT APPLE)
        target_link_libraries(my_app pthread dl)
elseif(APPLE)
        target_link_libraries(my_app pthread)
elseif(WIN32)
        target_link_libraries(my_app ws2_32 userenv)
        # Use static runtime on Windows
        set_property(TARGET my_app PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
endif()
```

### Makefile Integration

```makefile
# Makefile
CC = gcc
CFLAGS = -Wall -Wextra -std=c99 -O2
LDFLAGS = -L./target/release -lemissary_c -static

# Platform-specific flags
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
        LDFLAGS += -lpthread -ldl
endif
ifeq ($(UNAME_S),Darwin)
        LDFLAGS += -lpthread
endif

my_app: main.o
    $(CC) -o $@ $^ $(LDFLAGS)

main.o: main.c emissary-c.h
    $(CC) $(CFLAGS) -c $<

clean:
    rm -f *.o my_app
```

### pkg-config Support

Create `emissary-c.pc`:

```ini
prefix=/usr/local
libdir=${prefix}/lib
includedir=${prefix}/include

Name: emissary-c
Description: C bindings for Emissary I2P router
Version: 0.1.0
Libs: -L${libdir} -lemissary_c -lpthread -ldl
Libs.private: -static
Cflags: -I${includedir}
```

Then use in your build:

```bash
gcc main.c $(pkg-config --cflags --libs --static emissary-c) -o my_app
```

## Troubleshooting

### Build Issues

**Rust not found:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**OpenSSL errors on Linux:**
```bash
# Install OpenSSL development headers
sudo apt install libssl-dev  # Ubuntu/Debian
sudo dnf install openssl-devel  # Fedora/CentOS
```

**Static linking errors:**
- Ensure you have static versions of system libraries
- On Linux, install `glibc-static` or equivalent packages
- Use `-static-libgcc -static-libstdc++` for GCC
- Check for missing static dependencies with `nm` or `objdump`

### Runtime Issues

**Static binary advantages:**
- No "library not found" errors
- Self-contained executable
- No need to manage library paths

**Large binary size:**
- Expected with static linking
- Use `-Os` or `-Oz` for size optimization
- Strip debug symbols with `strip` command

**Router startup failures:**
- Check firewall settings (I2P needs network access)
- Ensure sufficient memory (at least 64MB recommended)
- Check logs for specific error messages

## Cross-Compilation

### Linux to Windows

```bash
# Install cross-compilation toolchain
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --package emissary-c --release --target x86_64-pc-windows-gnu
```

### macOS to Linux

```bash
# Install cross-compilation toolchain
rustup target add x86_64-unknown-linux-gnu

# Build for Linux (requires Linux toolchain)
cargo build --package emissary-c --release --target x86_64-unknown-linux-gnu
```

## Environment Variables

- `RUST_LOG`: Set to `debug` or `trace` for detailed logging
- `CARGO_TARGET_DIR`: Override build output directory
- `EMISSARY_DATA_DIR`: Override default data directory (if supported)

## Testing the Build

Run the provided example to verify everything works:

```bash
cd emissary-c

# Compile the example (static linking)
gcc examples/simple_usage.c -L../target/release -lemissary_c -lpthread -ldl -static -o test_example

# Run it
./test_example
```

Expected output should show router initialization, startup, SAMv3 port information, and clean shutdown.

