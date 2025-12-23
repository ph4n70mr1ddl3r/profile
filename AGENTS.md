# Agent Development Guidelines

## Build & Test Commands
- Build all: `cargo build --manifest-path profile-root/Cargo.toml`
- Build release: `cargo build --release --manifest-path profile-root/Cargo.toml`
- Run all tests: `cargo test --manifest-path profile-root/Cargo.toml`
- Run single test: `cargo test --manifest-path profile-root/Cargo.toml -- test_name`
- Run tests in specific crate: `cargo test -p profile-server`
- Run integration tests only: `cargo test --test integration_test_name`
- Format code: `cargo fmt --manifest-path profile-root/Cargo.toml`
- Check lint: `cargo clippy --manifest-path profile-root/Cargo.toml`

## Code Style Guidelines

**Imports**: Order as: std → external crates → local modules. Use `use crate::` for intra-crate references.
**Formatting**: Use `cargo fmt`. Prefer 4-space indentation (default Rust). Lines ≤100 chars where possible.
**Types**: Strong typing with descriptive aliases (e.g., `type PrivateKey = Zeroizing<Vec<u8>>`). Use `Arc<RwLock<T>>` for shared concurrent state.
**Naming**: snake_case for functions/variables, PascalCase for types/modules. SCREAMING_SNAKE_CASE for constants.
**Error Handling**: Define custom error enums with `#[derive(Debug, Clone)]`, implement `Display` and `std::error::Error`. Use `Result<T, Error>` consistently.
**Documentation**: Add module docs with `//!` and function docs with `///`. Include purpose and usage examples.
**Tests**: Place unit tests in `#[cfg(test)]` modules within source files. Integration tests go in `tests/` directories. Use `#[tokio::test]` for async tests. Test names should be descriptive (`test_function_behavior_expected`).
**Async**: Always use tokio runtime. Mark async functions with `async fn`, await with `.await`.
**Memory Safety**: Wrap sensitive data (private keys) in `zeroize::Zeroizing`. Call `.zeroize()` before dropping sensitive byte arrays.
