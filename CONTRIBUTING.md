# Contributing to Claims Toolkit

Thank you for your interest in contributing! This document provides guidelines for getting started.

## Getting Started

### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- Docker (optional, for containerized development)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/KidIkaros/claims-toolkit.git
cd claims-toolkit

# Build all crates
cargo build --release

# Run tests
cargo test --release

# Run the CLI
cargo run --release -p claims-toolkit-cli -- --help
```

### Docker Development

```bash
# Build and run with Docker Compose
docker-compose up -d dev

# Execute commands in the container
docker-compose exec claims-toolkit claims-toolkit --help

# Parse a file through Docker
docker-compose exec -T claims-toolkit claims-toolkit parse /data/sample.835
```

## Project Structure

```
crates/
├── era835/              # X12 835 ERA parser
├── era835-synth/        # Synthetic 835 generator
├── phi-scan/            # PHI detection
├── claims-scrub/        # Claims validation
├── claims-837/          # X12 837 claim parser (NEW)
└── claims-toolkit-cli/  # Unified CLI
```

## Contribution Workflow

1. **Fork and Branch**: Create a feature branch from `main`
2. **Make Changes**: Follow coding standards below
3. **Test**: Add tests for new functionality
4. **Document**: Update README and relevant docs
5. **Submit PR**: Use the PR template

### Coding Standards

- **Rust**: Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Address all `cargo clippy` warnings
- **Documentation**: Add rustdoc comments for public APIs
- **Tests**: Aim for >80% coverage for new code

### Commit Messages

Use conventional commits format:

```
feat: add X12 837 parser for professional claims
fix: resolve panic on malformed CAS segment
docs: update README with new examples
perf: optimize PHI detection with Aho-Corasick
```

## Areas for Contribution

### Good First Issues

- Add more CARC/RARC code definitions
- Improve error messages with context
- Add output format options (YAML, TOML)
- Write additional documentation examples

### Feature Areas

- **Parsers**: Additional X12 transaction sets (834, 276/277, etc.)
- **Outputs**: New export formats (PDF reports, HTML dashboards)
- **Integrations**: Webhook support, cloud storage connectors
- **Performance**: Benchmark and optimize hot paths

## Testing

### Running Tests

```bash
# All tests
cargo test --release

# Specific crate
cargo test --release -p era835

# With output
cargo test --release -- --nocapture
```

### Adding Tests

- **Unit tests**: In `src/` files, test individual functions
- **Integration tests**: In `tests/` directories, test full workflows
- **Doc tests**: In rustdoc comments, show usage examples

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_835() {
        let input = "ISA*00*...";
        let result = parse_era835(input);
        assert!(result.is_ok());
    }
}
```

## Documentation

- **Code**: Add rustdoc comments (`///`) for public items
- **Examples**: Add real-world usage to `examples/` directory
- **README**: Update if adding new commands or features
- **Changelog**: Add entry under `## Unreleased`

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release date
3. Tag: `git tag -a v0.3.0 -m "Release v0.3.0"`
4. Push tag: `git push origin v0.3.0`
5. CI automatically builds and publishes

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be respectful, constructive, and inclusive.

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (Apache-2.0 OR OPL-1.1).
