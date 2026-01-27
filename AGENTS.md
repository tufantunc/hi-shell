# AGENTS.md

## Commands

### Building
- **Release build**: `cargo build --release`
- **macOS with Metal acceleration**: `cargo build --release --features metal`
- **Linux with CUDA support**: `cargo build --release --features cuda`
- **Debug build**: `cargo build`
- **Quick check**: `cargo check`

### Testing
- **Run all tests**: `cargo test`
- **Run single test**: `cargo test test_name` (e.g., `cargo test test_parse_llm_response_json`)
- **Run tests with output**: `cargo test -- --nocapture`
- **Run specific test file**: `cargo test --test llm_tests`

### Code Quality
- **Lint**: `cargo clippy` (fix warnings with `cargo clippy --fix`)
- **Format**: `cargo fmt`
- **Check without building**: `cargo check`

## Architecture

### Project Structure
- CLI tool that translates natural language → shell commands via LLM
- `src/main.rs` - CLI entry point, REPL, command execution
- `src/lib.rs` - Module exports (minimal)
- `src/config.rs` - Configuration management with TOML serialization
- `src/error.rs` - Custom error type using `thiserror`
- `src/telemetry.rs` - Anonymous usage tracking
- `src/llm/mod.rs` - Core LLM traits, utilities, JSON parsing
- `src/llm/cloud.rs` - Cloud LLM clients (OpenRouter, Gemini, Anthropic, OpenAI)
- `src/llm/local.rs` - Local LLM clients (Ollama, LM Studio)
- `src/llm/embedded.rs` - Embedded models using candle/GGUF
- `tests/` - Integration tests (config_tests, error_tests, llm_tests)

### Key Traits
- `LlmBackend` - Trait for LLM providers, implemented by all backends
- All async traits use `#[async_trait]` macro
- Error handling: `thiserror` for library errors, `anyhow::Result` for application code

## Code Style

### Rust Conventions
- **Edition**: Rust 2024
- **Async**: tokio runtime with `#[tokio::main]` for entry points
- **Error handling**: Use `anyhow::Result` in main/CLI code, `thiserror` for library error types
- **Traits**: All trait methods are async and require `Send + Sync` bounds
- **Result type**: Defined in `src/error.rs` as `pub type Result<T> = std::result::Result<T, HiShellError>`

### Import Ordering
Organize imports in this order with blank lines between groups:
```rust
// 1. std library
use std::io::{self, Read, Write};
use std::path::PathBuf;

// 2. external crates
use anyhow::Result;
use clap::Parser;
use tokio::time::Duration;

// 3. internal modules
use crate::config::{Config, LlmProvider};
use crate::error::{HiShellError, Result};
```

### Naming Conventions
- **Modules**: `snake_case` (e.g., `llm/mod.rs`, `config.rs`)
- **Functions**: `snake_case` (e.g., `generate_command`, `get_system_info`)
- **Types**: `PascalCase` (e.g., `Config`, `CommandResponse`, `HiShellError`)
- **Enums**: `PascalCase` variants (e.g., `LlmProvider::Embedded`, `LocalProviderType::Ollama`)
- **Constants**: `SCREAMING_SNAKE_CASE` (rarely used)
- **Test functions**: `test_<feature>_<scenario>` (e.g., `test_parse_llm_response_json`)

### Error Handling
- Use `#[from]` attribute for automatic error conversions from external types
- Use `ok_or_else(|| HiShellError::Variant(msg))` for Option → Result conversions
- Use `map_err(|e| HiShellError::Variant(e.to_string()))` for manual conversions
- Always use `?` operator for error propagation (avoid `unwrap()` except in tests)
- Custom error types in `src/error.rs` use thiserror derive macro

### Traits and Implementations
- Implement `LlmBackend` trait for all LLM providers
- Use `#[async_trait]` macro for async traits
- Common derives: `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]`
- Trait objects: `Box<dyn LlmBackend>` for runtime polymorphism
- All trait methods accept `&self` and return `Result<T>`

### Async/Await Patterns
- Entry point: `#[tokio::main] async fn main() -> anyhow::Result<()>`
- Separate main (error handling) from run (business logic)
- Use `#[async_trait]` for trait definitions with async methods
- Set network timeouts: `reqwest::Client::builder().timeout(Duration::from_secs(30))`
- Background tasks: `tokio::spawn(async move { ... })`
- Clone config before moving into async contexts: `config.clone()`

### Configuration and Serialization
- Config struct uses `#[serde(default)]` for optional fields
- Use `directories` crate for config file paths (cross-platform)
- Implement `Default` trait for sensible defaults
- TOML format: `toml::to_string_pretty()` for saving, `toml::from_str()` for loading
- Use `Option<T>` for fields that may be unset

### Logging
- Use `tracing` crate with `tracing_subscriber` for initialization
- Log levels: `debug!()`, `info!()`, `warn!()`, `error!()`
- Conditional logging based on CLI verbose flag: `env_filter = "hi_shell=debug"` or `"hi_shell=info"`
- Initialize once in `run()` before other operations

### Testing Conventions
- Integration tests in `tests/` directory (one file per module)
- Simple synchronous tests with `#[test]` attribute
- Use `.unwrap()` in tests (acceptable in test context)
- Assert with `assert_eq!()`, `assert!()`, `assert!(!res.dangerous)` for safe commands
- Test names: `test_<functionality>_<scenario>`

### CLI Patterns
- Use `clap` derive macro: `#[derive(Parser, Debug)]`
- Add attributes: `#[command(name = "hi-shell")]`, `#[command(version = env!("CARGO_PKG_VERSION"))]`
- Args: `#[arg(long, help = "...")]`, `#[arg(short, long)]`
- Trailing args: `#[arg(trailing_var_arg = true, allow_hyphen_values = true)]`

### HTTP Client Patterns
- Use `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()?`
- Check response status: `if !response.status().is_success() { return Err(...) }`
- Parse JSON: `let json: serde_json::Value = response.json().await?`
- Extract fields: `json["field"].as_str().map(|s| s.to_string())`

### Feature Flags
- `metal` - macOS Metal acceleration: `--features metal`
- `cuda` - CUDA support: `--features cuda`
- Default features: empty in Cargo.toml

### Conditional Compilation
```rust
#[cfg(windows)]
// Windows-specific code

#[cfg(not(windows))]
// Unix-specific code
```

### Common Derives
- structs/enums: `Debug, Clone, Serialize, Deserialize`
- config structs: add `PartialEq` for comparison tests
- error enums: `Error` from thiserror

### Platform-Specific Code
- Command execution: `cmd.exe` on Windows, `sh` on Unix
- Config paths: use `directories` crate for cross-platform support
- Acceleration: Metal on macOS, CUDA on Linux, CPU fallback

### LLM Backend Implementation Pattern
All backends implement `LlmBackend` trait with:
1. `new(config: Config) -> Self` constructor
2. `generate_command(messages: &[Message], repair_context: Option<&str>) -> Result<CommandResponse>`
3. Use `crate::llm::get_system_prompt(repair_context)` for system prompt
4. Use `crate::llm::parse_llm_response()` to parse LLM output
5. Use `crate::llm::create_spinner()` for progress indication
