# Configuration Reference

`spell_check` is configured via a TOML file, usually named `spellcheck.toml`.

## `[files]` Section
Controls which files are scanned.

- **`include`**: A list of glob patterns for files to include.
  - Default: `["**/*.{md,txt,rs,js,ts,py,c,cpp,h,hpp,go,java}"]`
- **`exclude`**: A list of glob patterns for files to ignore.
  - Example: `["target/**", "node_modules/**"]`

## `[dictionary]` Section
Extends the built-in dictionary.

- **`extra_words`**: A list of words to allow globally. These are case-insensitive.
- **`extra_dictionaries`**: A list of paths to plain-text files containing one word per line.

## `[ignore]` Section
Fine-tuned control over what is ignored.

- **`words`**: A list of words to ignore during the check (similar to `extra_words`).

## Sample Configuration (`spellcheck.toml`)

```toml
[files]
include = ["src/**/*.rs", "*.md"]
exclude = ["target/**", "build/**"]

[dictionary]
extra_words = ["rust", "tokio", "serde", "anyhow"]
extra_dictionaries = ["./custom_words.txt"]

[ignore]
words = ["TODO", "FIXME", "XXX"]
```
