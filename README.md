# hi-shell 🐚

👋 **hi-shell**: An intelligent terminal assistant that translates your natural language descriptions into executable bash commands.

`hi-shell` helps you bridge the gap between "what I want to do" and "how do I write that command?". Whether you're a terminal veteran or a newcomer, `hi-shell` provides a fast, AI-powered way to generate and execute commands safely.

## ✨ Features

- **Multi-LLM Support**:
  - **Embedded**: Run models locally using `candle` (e.g., Phi-3-mini) with hardware acceleration (Metal/CUDA).
  - **Local**: Connect to your own Ollama or LM Studio instance.
  - **Cloud**: Integration with OpenRouter, Gemini, and Anthropic.
- **Interactive REPL**: A dedicated shell environment for continuous assistance.
- **One-shot Mode**: Get quick answers directly from your command line.
- **Safety First**: Dangerous commands are flagged, and confirmation is required before execution.
- **Telemetry**: Optional anonymous usage stats to help improve the tool.

## 🚀 Installation

You can install `hi-shell` using your preferred method:

### ⚡ Quick Install (macOS & Linux)
```bash
curl -sSL https://raw.githubusercontent.com/tufantunc/hi-shell/main/install.sh | bash
```

### 🍏 Homebrew (macOS & Linux)
```bash
brew tap tufantunc/tap
brew install hi-shell
```

### 🪟 Scoop (Windows)
```powershell
scoop bucket add hi-shell https://github.com/tufantunc/scoop-bucket
scoop install hi-shell
```

### 🦀 Cargo
If you have Rust installed, you can install it via Cargo:
```bash
# Install via crates.io
cargo install hi-shell

# OR install pre-compiled binary via cargo-binstall
cargo binstall hi-shell
```

### 📦 Manual Download
You can download the pre-built binaries for your operating system from the [Releases](https://github.com/tufantunc/hi-shell/releases) page.

1. Download the version corresponding to your OS (macOS, Linux, or Windows).
2. Move the binary to a folder in your `PATH`.
3. Run `hi-shell --init` to set up your preferred LLM provider.

## 🛠 Development

If you'd like to contribute or build from source, follow these steps.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- C Compiler (for some dependencies)

### Cloning the Repository

```bash
git clone https://github.com/tufantunc/hi-shell.git
cd hi-shell
```

### Building from Source

You can build the project for your specific platform using Cargo.

#### 🍏 macOS (with Metal support)
```bash
cargo build --release --features metal
```

#### 🐧 Linux
```bash
cargo build --release
```

#### 𝝿 Linux (with CUDA support)
```bash
cargo build --release --features cuda
```

#### 🪟 Windows
```bash
cargo build --release
```

The compiled binary will be located in `target/release/hi-shell`.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve `hi-shell`.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
