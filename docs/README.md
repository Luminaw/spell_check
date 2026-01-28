# Spell Check Documentation

Welcome to the documentation for **Spell Check**, a high-performance, memory-safe spell-checking tool designed for developers.

## Contents

- [Usage Guide](usage.md): How to get started and run checks.
- [Configuration Reference](configuration.md): Detailed explanation of all configuration options.
- [Architecture Overview](architecture.md): Insights into how the engine works.

## Quick Start

1. **Install**: Ensure you have Rust installed and run `cargo build --release`.
2. **Initialize**: Run `spell_check init` in your project root to create a `spellcheck.toml`.
3. **Run**: Run `spell_check check` to start scanning your project.

## Features

- **Blazing Fast**: Uses Rust's concurrency primitives to scan thousands of files in seconds.
- **Memory Safe**: 100% safe Rust code with zero unsafe blocks.
- **Highly Configurable**: Comprehensive glob pattern support and custom dictionaries.
- **Embedded Dictionary**: Comes with a large default dictionary (370k+ words).
- **Silent & Clean**: Integrates into CI/CD pipelines effortlessly.
