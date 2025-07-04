# Contributing to Longfellow-ZK Rust

Thank you for your interest in contributing to the Rust implementation of Longfellow-ZK!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/longfellow-zk-rust.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run benchmarks: `cargo bench`
7. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- Clang++ (for C++ comparison tests)
- CMake 3.13+ (for building C++ library)

### Building

```bash
# Build all packages
cargo build

# Build in release mode
cargo build --release

# Build with all features
cargo build --all-features
```

## Code Style

We use `rustfmt` and `clippy` for code formatting and linting:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Run clippy with all targets
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p longfellow-algebra

# Run tests with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Writing Tests

- Add unit tests in the same file as the code using `#[cfg(test)]`
- Add integration tests in `tests/` directory
- Use property-based testing with `proptest` for complex invariants
- Ensure all tests pass before submitting PR

### Equivalence Testing

When modifying existing functionality:

1. Run equivalence tests against C++ implementation
2. Ensure bit-for-bit compatibility
3. Document any intentional deviations

```bash
cargo test -p longfellow-equivalence-tests
```

## Benchmarking

Before submitting performance-related changes:

1. Run benchmarks before and after changes
2. Include benchmark results in PR description
3. Ensure no performance regressions

```bash
# Run benchmarks
cargo bench

# Compare with baseline
cargo bench -- --baseline master
```

## Documentation

- Add doc comments to all public APIs
- Include examples in doc comments
- Run `cargo doc --open` to preview documentation
- Ensure no doc warnings: `cargo doc --no-deps`

## Pull Request Process

1. **Title**: Use clear, descriptive titles
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `perf:` for performance improvements
   - `refactor:` for code refactoring
   - `test:` for test additions/changes

2. **Description**: Include:
   - What changes were made
   - Why these changes were necessary
   - Any breaking changes
   - Benchmark results (if applicable)

3. **Checklist**:
   - [ ] Tests pass (`cargo test`)
   - [ ] Code formatted (`cargo fmt`)
   - [ ] Clippy clean (`cargo clippy`)
   - [ ] Documentation updated
   - [ ] Benchmarks run (if applicable)
   - [ ] Equivalence tests pass (if modifying existing functionality)

## Security

- Never commit secrets or private keys
- Report security vulnerabilities privately
- Follow cryptographic best practices
- Ensure constant-time operations where required

## Adding New Modules

When adding a new module:

1. Create new crate in workspace
2. Add to `Cargo.toml` workspace members
3. Follow existing module structure
4. Add comprehensive tests
5. Add benchmarks
6. Update main README

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).