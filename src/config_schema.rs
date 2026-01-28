//! Configuration schema for the spell checker.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The root configuration structure for `spell_check`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// File scanning configuration.
    #[serde(default)]
    pub files: FilesConfig,
    /// Dictionary management configuration.
    #[serde(default)]
    pub dictionary: DictionaryConfig,
    /// Custom ignore settings.
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

/// Configuration for controlling which files are scanned.
#[derive(Debug, Serialize, Deserialize)]
pub struct FilesConfig {
    /// List of glob patterns to include in the scan.
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    /// List of glob patterns to exclude from the scan.
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_include() -> Vec<String> {
    vec!["**/*.{md,txt,rs,js,ts,py,c,cpp,h,hpp,go,java}".to_string()]
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            exclude: Vec::new(),
        }
    }
}

/// Configuration for the dictionary and custom word lists.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryConfig {
    /// List of additional words to allow globally.
    #[serde(default)]
    pub extra_words: Vec<String>,
    /// Paths to external line-separated dictionary files.
    #[serde(default)]
    pub extra_dictionaries: Vec<PathBuf>,
}

/// Configuration for words to ignore during spelling checks.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IgnoreConfig {
    /// List of specific words to ignore (case-insensitive).
    #[serde(default)]
    pub words: Vec<String>,
}
