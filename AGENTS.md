# AGENTS.md

## Commands
- Build: `cargo build --release` (add `--features metal` for macOS, `--features cuda` for CUDA)
- Test all: `cargo test`
- Test single: `cargo test test_name` (e.g., `cargo test test_parse_llm_response_json`)
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Check: `cargo check`

## Architecture
- CLI tool: natural language → shell commands via LLM
- `src/main.rs` - CLI entry, REPL, command execution
- `src/llm/` - LLM backends: `embedded.rs` (candle/GGUF), `local.rs` (Ollama/LM Studio), `cloud.rs` (OpenRouter/Gemini/Anthropic)
- `src/config.rs` - Configuration management
- `src/telemetry.rs` - Anonymous usage tracking
- `tests/` - Integration tests

## Code Style
- Rust 2024 edition, async/await with tokio
- Use `anyhow::Result` for errors, `thiserror` for custom error types
- Imports: std first, external crates, then internal modules
- Use `tracing` for logging, `clap` derive for CLI args
- Follow existing patterns for new LLM backends (implement `LlmBackend` trait)
