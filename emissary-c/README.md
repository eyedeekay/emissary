# Emissary C Bindings

A minimal C Foreign Function Interface (FFI) library for the Emissary I2P router, providing essential lifecycle control and SAMv3 API bridge access for C applications.

## Features

- **Minimal API Surface**: Only essential functions for router lifecycle management
- **Memory Safe**: All FFI boundaries protected with panic catching and null pointer checks
- **Thread Safe**: Status and information functions are thread-safe
- **SAMv3 Integration**: Automatic SAMv3 API bridge setup with port discovery
- **Cross Platform**: Builds on Linux, macOS, and Windows
- **Zero Configuration**: Sensible defaults with no complex setup required
- **Robust Error Handling**: Comprehensive error codes and status reporting
- **Production Ready**: Designed for embedding in production applications

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- C compiler (GCC, Clang, or MSVC)
- Git

### Building the Library

```bash
# Clone the repository
git clone https://github.com/eyedeekay/emissary
cd emissary

# Build the C library (creates both static and dynamic libraries)
cargo build --package emissary-c --release

# The libraries will be available in:
# target/release/libemissary_c.a     (static library)
# target/release/libemissary_c.so    (dynamic library - Linux)
# target/release/libemissary_c.dylib (dynamic library - macOS)
# target/release/emissary_c.dll      (dynamic library - Windows)
```

### Using the Library

1. **Copy the header file**: `emissary-c.h` to your project
2. **Link the library**: Add the static or dynamic library to your build
3. **Include the header**: `#include "emissary-c.h"`

#### Example Makefile

```makefile
# Linux/macOS example
CC = gcc
CFLAGS = -Wall -Wextra -std=c99
LDFLAGS = -L./target/release -lemissary_c -lpthread -ldl

example: example.c emissary-c.h
	$(CC) $(CFLAGS) -o example example.c $(LDFLAGS)

clean:
	rm -f example
```

#### CMake Example

```cmake
cmake_minimum_required(VERSION 3.10)
project(emissary_example)

set(CMAKE_C_STANDARD 99)

# Find the emissary-c library
find_library(EMISSARY_C_LIB emissary_c PATHS ${CMAKE_SOURCE_DIR}/target/release)

add_executable(example example.c)
target_link_libraries(example ${EMISSARY_C_LIB})

# Platform-specific system libraries
if(UNIX AND NOT APPLE)
    target_link_libraries(example pthread dl)
elseif(APPLE)
    target_link_libraries(example pthread)
elseif(WIN32)
    target_link_libraries(example ws2_32 userenv)
endif()
```

## API Reference

### Core Functions

```c
// Initialize router instance
emissary_router_t* emissary_init(void);

// Start router operations (non-blocking)
int emissary_start(emissary_router_t* router);

// Stop router operations (non-blocking)
int emissary_stop(emissary_router_t* router);

// Free all resources
void emissary_destroy(emissary_router_t* router);
```

### Status Functions

```c
// Get current router status
int emissary_get_status(emissary_router_t* router);

// Check if SAMv3 is available
int emissary_sam_available(emissary_router_t* router);

// Get SAMv3 port numbers
int emissary_get_sam_tcp_port(emissary_router_t* router);
int emissary_get_sam_udp_port(emissary_router_t* router);
```

### Error Codes

| Code | Constant | Description |
|------|----------|-------------|
| 0 | `EMISSARY_SUCCESS` | Operation completed successfully |
| -1 | `EMISSARY_ERROR_GENERIC` | Unspecified failure |
| -2 | `EMISSARY_ERROR_INVALID_PARAM` | NULL pointer or invalid parameter |
| -3 | `EMISSARY_ERROR_NOT_INITIALIZED` | Router handle is invalid |
| -4 | `EMISSARY_ERROR_ALREADY_STARTED` | Router is already running |
| -5 | `EMISSARY_ERROR_NOT_STARTED` | Router is not currently running |
| -6 | `EMISSARY_ERROR_NETWORK` | Network configuration failure |
| -7 | `EMISSARY_ERROR_RESOURCE` | Insufficient system resources |

### Status Codes

| Code | Constant | Description |
|------|----------|-------------|
| 0 | `EMISSARY_STATUS_STOPPED` | Router is stopped |
| 1 | `EMISSARY_STATUS_STARTING` | Router is starting up |
| 2 | `EMISSARY_STATUS_RUNNING` | Router is running and ready |
| 3 | `EMISSARY_STATUS_STOPPING` | Router is shutting down |
| 4 | `EMISSARY_STATUS_ERROR` | Router is in an error state |

## Usage Example

```c
#include <stdio.h>
#include <unistd.h>
#include "emissary-c.h"

int main(void) {
    // Initialize router
    emissary_router_t* router = emissary_init();
    if (router == NULL) {
        fprintf(stderr, "Failed to initialize router\n");
        return 1;
    }

    // Start router
    int result = emissary_start(router);
    if (result != EMISSARY_SUCCESS) {
        fprintf(stderr, "Failed to start router: %d\n", result);
        emissary_destroy(router);
        return 1;
    }

    // Wait for router to become operational
    int status;
    do {
        status = emissary_get_status(router);
        if (status == EMISSARY_STATUS_STARTING) {
            printf("Router starting...\n");
            sleep(1);
        }
    } while (status == EMISSARY_STATUS_STARTING);

    if (status != EMISSARY_STATUS_RUNNING) {
        fprintf(stderr, "Router failed to start\n");
        emissary_destroy(router);
        return 1;
    }

    // Get SAMv3 port
    if (emissary_sam_available(router)) {
        int sam_port = emissary_get_sam_tcp_port(router);
        printf("SAMv3 available on port: %d\n", sam_port);
        printf("Connect to 127.0.0.1:%d for I2P API\n", sam_port);
    }

    // Router operations here...
    sleep(10);

    // Clean shutdown
    emissary_stop(router);
    emissary_destroy(router);
    return 0;
}
```

## Build Platform Support

### Linux

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev
cargo build --package emissary-c --release

# Link with: -lemissary_c -lpthread -ldl
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install
cargo build --package emissary-c --release

# Link with: -lemissary_c -lpthread
```

### Windows

```cmd
# Install MSVC Build Tools or Visual Studio
# Install Rust for Windows
cargo build --package emissary-c --release

# Link with: emissary_c.lib ws2_32.lib userenv.lib
```

## Thread Safety

- **Safe**: `emissary_init()`, `emissary_get_status()`, `emissary_sam_available()`, `emissary_get_sam_*_port()`
- **Unsafe**: `emissary_start()`, `emissary_stop()`, `emissary_destroy()` - Do not call concurrently on the same handle

## Memory Management

- Router handles are allocated by `emissary_init()` and must be freed with `emissary_destroy()`
- No other memory management is required from the C side
- All internal resources are automatically cleaned up on destruction

## Advanced Configuration

The C bindings use sensible defaults:

- **Transport**: NTCP2 on random port
- **Transit Tunnels**: Disabled (minimal resource usage)  
- **SAMv3 Bridge**: Enabled on random ports
- **Data Directory**: System temporary directory
- **Network**: I2P mainnet (netId=2)

For advanced configuration, modify the source code or use the native Rust API directly.

## Troubleshooting

### Common Issues

1. **Library not found**: Ensure the library path is correct in your linker flags
2. **Router fails to start**: Check network connectivity and available ports
3. **Segmentation fault**: Ensure proper error checking and null pointer validation
4. **Build fails**: Verify Rust toolchain and system dependencies are installed

### Debug Build

```bash
# Build with debug symbols
cargo build --package emissary-c

# Enable Rust logging (requires rebuilding with log features)
export RUST_LOG=debug
```

### Platform-Specific Notes

- **Linux**: Requires `libssl-dev` and `pkg-config`
- **macOS**: May require explicit OpenSSL paths if using Homebrew
- **Windows**: Requires Visual Studio Build Tools or full Visual Studio

## Contributing

1. Follow the existing code style and API design principles
2. Add tests for new functionality
3. Update documentation for API changes
4. Ensure cross-platform compatibility

## License

This project is licensed under the MIT License - see the LICENSE file for details.
