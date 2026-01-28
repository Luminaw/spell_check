use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub dictionary: DictionaryConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryConfig {
    #[serde(default)]
    pub extra_words: Vec<String>,
    #[serde(default)]
    pub extra_dictionaries: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub words: Vec<String>,
}
