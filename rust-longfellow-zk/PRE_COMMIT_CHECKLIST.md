# Pre-Commit Checklist

Before committing the Rust implementation, ensure:

## Code Quality
- [ ] All code follows Rust idioms and best practices
- [ ] No `unsafe` code without justification
- [ ] All public APIs have documentation
- [ ] Error handling uses `Result` types appropriately
- [ ] No panics in library code (only in tests)

## Testing
- [ ] Unit tests for all modules
- [ ] Integration tests for cross-module functionality
- [ ] Equivalence tests comparing with C++ implementation
- [ ] Property-based tests for complex invariants
- [ ] All tests pass: `cargo test`

## Performance
- [ ] Benchmarks show comparable or better performance than C++
- [ ] No performance regressions
- [ ] Memory usage is reasonable
- [ ] Parallel operations use Rayon effectively

## Documentation
- [ ] README.md is comprehensive
- [ ] BENCHMARKING.md explains benchmark usage
- [ ] CONTRIBUTING.md provides clear guidelines
- [ ] All public APIs have rustdoc comments
- [ ] Examples are provided for complex functionality

## Build & CI
- [ ] `cargo build --release` succeeds
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo doc --no-deps` builds without warnings
- [ ] GitHub Actions workflow is configured

## Security
- [ ] No hardcoded secrets or keys
- [ ] Cryptographic operations are constant-time where needed
- [ ] Dependencies are from trusted sources
- [ ] `cargo audit` shows no vulnerabilities

## Repository Structure
- [ ] .gitignore excludes all build artifacts
- [ ] License files are included
- [ ] rust-toolchain.toml specifies Rust version
- [ ] Cargo.toml files have correct metadata

## Final Steps
1. Review all changes: `git diff`
2. Stage files: `git add rust-longfellow-zk/`
3. Verify staged files: `git status`
4. Commit with descriptive message
5. Push to feature branch
6. Create pull request with comprehensive description

## Notes
- The C++ library must be built for equivalence tests to work
- Benchmarks require release mode for accurate results
- Some tests may be slow in debug mode