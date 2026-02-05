# Contributing to Muxis

Thank you for your interest in contributing to Muxis! We welcome contributions from the community to help make this the best Redis client for Rust.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally.
3.  **Install Rust**: Ensure you have a recent version of Rust installed (1.83+ required).

## Development Workflow

We follow a standard GitHub pull request workflow.

1.  **Create a branch** for your feature or fix:
    ```bash
    git checkout -b feat/your-feature-name
    # or
    git checkout -b fix/your-bug-fix
    ```
2.  **Make your changes**. Please follow the code style guidelines below.
3.  **Run tests** to ensure no regressions:
    ```bash
    cargo test --all-features
    ```
4.  **Format and lint** your code:
    ```bash
    cargo fmt --all
    cargo clippy --all-targets --all-features -- -D warnings
    ```
5.  **Commit your changes** using [Conventional Commits](https://www.conventionalcommits.org/):
    ```bash
    git commit -m "feat(core): add new connection option"
    ```
6.  **Push to your fork** and open a Pull Request.

## Code Style

- **Rustfmt**: We use standard `rustfmt` settings. Run `cargo fmt` before committing.
- **Clippy**: Code must pass `cargo clippy` with no warnings.
- **Documentation**: All public APIs must be documented. Run `cargo doc --open` to verify.
- **No Emojis**: Please avoid using emojis in code comments or documentation files (except for this guide if necessary).

## Project Structure

- `muxis-core`: Core connection handling and multiplexing logic.
- `muxis-proto`: RESP protocol implementation (codec, frames).
- `muxis-client`: High-level public API.
- `muxis-cluster`: Cluster support (in progress).

## Testing

- **Unit Tests**: Place unit tests in the same file or a `tests` module within the source file.
- **Integration Tests**: Place integration tests in the `tests/` directory of the respective crate.
- **Docker**: For tests requiring a real Redis instance, use the provided Docker Compose setup (if available) or run a local Redis instance. Run these tests with `--ignored` flag if they are marked as such.

## License

By contributing, you agree that your contributions will be licensed under the project's dual MIT/Apache-2.0 license.
