# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-02-05

### Added

- **Workspace Setup**: Initial workspace with 5 crates:
  - `muxis-proto`: RESP protocol codec
  - `muxis-core`: Core connection handling
  - `muxis-cluster`: Cluster support
  - `muxis-client`: Public API
  - `muxis-test`: Test utilities

- **CI/CD**: GitHub Actions workflow with:
  - `cargo check`: Compilation verification
  - `cargo fmt`: Code formatting check
  - `cargo clippy`: Linting
  - `cargo test`: Unit tests
  - `cargo doc`: Documentation build

- **MSRV Policy**: Rust 1.78+ enforced via `rust-toolchain.toml`

- **Feature Flags**:
  - `tls`: TLS/SSL support (via tokio-tls)
  - `resp3`: RESP3 protocol support
  - `cluster`: Cluster mode support
  - `json`: RedisJSON commands
  - `streams`: Streams commands
  - `tracing`: Observability

- **Error Model**: Base error types in `muxis-proto`

- **Client Stubs**: Basic `Client` and `ClientPool` structs in `muxis-core`

### Fixed

- Dependency version resolution for tokio (1.40+)
- MSRV compatibility for native-tls (pinned to 0.2.11)
- Tokio-tls feature configuration

### Changed

- Switched from cargo-native-tls to tokio-tls for feature flag management
