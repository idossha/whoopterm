# Contributing to whoopterm

Thank you for your interest in contributing to whoopterm! We welcome contributions from the community.

## Getting Started

### Prerequisites
- Rust 1.70 or later
- Git
- Your favorite editor or IDE

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/whoopterm.git
   cd whoopterm
   ```
3. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Build for release
cargo build --release
```

## Code Style

- Use `rustfmt` for formatting code
- Use `clippy` for linting
- Follow Rust naming conventions

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Submit a pull request with a clear description of your changes
4. Wait for review and address any feedback

## Types of Contributions

### Bug Reports
- Check existing issues first
- Provide detailed steps to reproduce
- Include error messages and stack traces

### Feature Requests
- Explain the use case and benefits
- Consider implementation complexity

### Code Contributions
- Keep changes focused and atomic
- Add tests for new functionality
- Update documentation as needed

## Commit Messages

Use conventional commit format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for formatting changes
- `refactor:` for code refactoring
- `test:` for test additions
- `chore:` for maintenance tasks

## Security Vulnerabilities

If you discover a security vulnerability, please email the maintainer directly instead of creating a public issue.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.