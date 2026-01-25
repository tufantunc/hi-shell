# Contributing to hi-shell

Thank you for your interest in contributing to hi-shell! 🐚

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/hi-shell.git
   cd hi-shell
   ```
3. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- C Compiler (for native dependencies)

### Building

```bash
# macOS (with Metal acceleration)
cargo build --release --features metal

# Linux
cargo build --release

# Linux with CUDA
cargo build --release --features cuda

# Windows
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
cargo fmt      # Format code
cargo clippy   # Lint
cargo check    # Type check
cargo test     # Run tests
```

## Pull Request Process

1. Update documentation if you're changing functionality
2. Add tests for new features
3. Ensure all CI checks pass
4. Keep commits focused and atomic
5. Write clear commit messages

## Code Style

- Follow Rust conventions and idioms
- Use `anyhow::Result` for error handling
- Use `tracing` for logging
- Keep functions small and focused

## Adding a New LLM Backend

If you're adding a new LLM provider:

1. Create a new file in `src/llm/`
2. Implement the `LlmBackend` trait
3. Add configuration options to `src/config.rs`
4. Update the provider selection logic in `src/main.rs`
5. Document the new provider in README.md

## Reporting Issues

When reporting bugs, please include:

- Your operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce the issue
- Expected vs actual behavior
- Relevant error messages or logs

## Feature Requests

Feature requests are welcome! Please describe:

- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
