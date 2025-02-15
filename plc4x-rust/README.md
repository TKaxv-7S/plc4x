# PLC4X Rust Implementation

This project implements Rust bindings for Apache PLC4X, focusing on the Siemens S7 protocol. The implementation aims to provide:

-   Memory-safe, zero-copy protocol parsing using nom
-   Async/await support for modern Rust applications
-   High-performance industrial communication
-   Type-safe protocol implementation

## Current Status

-   [x] Initial project structure
-   [x] Basic S7 protocol types
-   [ ] Protocol parsing implementation
-   [ ] Connection handling
-   [ ] Async support
-   [ ] Testing infrastructure

## Getting Started

### Prerequisites

-   Rust (stable channel)
-   Cargo

### Installation

```bash
git clone https://github.com/apache/plc4x
cd plc4x-rust
cargo build
```

## Testing

### Unit Tests

```bash
cargo test
```

### Fuzz Testing

```bash
# Install cargo-fuzz (only needed once)
cargo install cargo-fuzz

# Run the fuzzer
cargo fuzz run header_parser
```

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and development process.
