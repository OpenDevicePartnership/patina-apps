# Patina Apps

A collection of EFI applications built with Rust for Patina UEFI environments.

## Quick Start

### Prerequisites

- Rust toolchain with the appropriate UEFI target (based on VM/hardware)

```bash
rustup target add [x86_64|aarch64|i686]-unknown-uefi
```

### Build

```bash
# Build all applications in release mode.
cargo make build

# Build all applications in debug mode.
cargo make build-debug

# Build specific app.
cargo make --env PACKAGE=<app-name> build-package
```

### Running

Applications will be built into `target/efi/<app-name>.efi`.
Copy the `.efi` to the system's drive (using USB drive or other methods) and run inside the UEFI shell.

## Applications

- **services_benchmark_test**: Benchmarks for core Patina service calls. Compares Rust timings to C.
