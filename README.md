# SpellCheck CLI

A high-performance, asynchronous CLI spellchecker written in Rust. Designed for production-level codebases with a focus on speed, flexibility, and a developer-friendly "NPM-style" configuration experience.

## Features

- **🚀 Concurrent Scanning**: Leverages `tokio` to scan thousands of files in parallel.
- **📁 Smart Traversal**: Respects `.gitignore` and `.spellcheckignore` files out of the box.
- **⚙️ Flexible Configuration**: Centralized `spellcheck.toml` with glob pattern support.
- **🚫 Inline Ignores**: Control checking directly in your code with `spellcheck-disable` comments.
- **📚 Embedded Dictionary**: High-quality English wordlist built-in (0-config ready).
- **🎨 Rich Output**: Beautifully highlighted terminal reports with context and line/column info.

## Installation

```bash
cargo install --path .
```

## Quick Start

1. Initialize a configuration in your project:
   ```bash
   spell_check init
   ```

2. Run the check:
   ```bash
   spell_check check .
   ```

## Configuration

### `spellcheck.toml`

Create a `spellcheck.toml` in your project root or pass one via `--config`.

```toml
[files]
include = ["src/**/*.rs", "docs/**/*.md"]
exclude = ["target/**", "tmp/**"]

[dictionary]
# Words to ignore for this project
extra_words = ["tokio", "serde", "anyhow", "globset"]
# Path to additional dictionary files (one word per line)
extra_dictionaries = ["./legal-terms.txt"]
```

### `.spellcheckignore`

Similar to `.gitignore`, use this file to exclude specific files or directories from all scans.

```text
# Ignore log files
*.log
# Ignore dependency directories
node_modules/
vendor/
```

## Inline Ignores

Disable spellchecking for specific parts of your file:

```rust
// spellcheck-disable
let secret_key = "awesomesauce-unspellable-token";
let api_endpoint = "https://internal.sys.srv";
// spellcheck-enable

fn main() {
    let x = "mispelledword"; // spellcheck-disable-line
}
```

## Advanced Usage

Check a specific project with a specific configuration:
```bash
spell_check --config ./my-config.toml check ./other-project
```

## Development

Run tests:
```bash
cargo test
```

Run integration tests against provided fixtures:
```bash
cargo test --test integration
```
