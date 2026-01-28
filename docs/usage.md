# Usage Guide

This guide covers the command-line interface and common workflows for using `spell_check`.

## Commands

### `check [PATH]`
Scans the specified path (defaults to the current directory) for spelling errors.

```bash
spell_check check .
```

- **Config Discovery**: By default, it looks for `spellcheck.toml` in the scan root.
- **Output**: Errors are printed with file path, line number, column, and context.
- **Exit Codes**: Returns `0` if no errors are found, or `1` if spelling errors or processing errors occur.

### `init`
Generates a default `spellcheck.toml` file in the current directory. Use this to quickly set up a new project.

```bash
spell_check init
```

## Global Options

- `--config <FILE>`, `-c <FILE>`: Specify a custom path to a configuration file.
- `--help`: Print help information.
- `--version`: Print version information.

## Inline Disabling

You can disable spell-checking for specific parts of your code using comments:

- `spellcheck-disable`: Disables checking from this point until the end of the file or until renabled.
- `spellcheck-enable`: Re-enables spell-checking.
- `spellcheck-disable-line`: Disables checking only for the current line.

### Example

```rust
// spellcheck-disable
let x = "some_very_long_non_word_string";
// spellcheck-enable

let y = "misspelled_word"; // spellcheck-disable-line
```
