use crate::config::Config;
use crate::dictionary::Dictionary;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use globset::{Glob, GlobSet, GlobSetBuilder};

pub struct Engine {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    #[allow(dead_code)]
    config: Arc<Config>,
    dictionary: Arc<Dictionary>,
    include_set: GlobSet,
    exclude_set: GlobSet,
}

#[derive(Debug)]
pub struct SpellError {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub word: String,
    pub context: String,
}

impl Engine {
    pub fn new(config: Config, dictionary: Dictionary) -> Self {
        let mut include_builder = GlobSetBuilder::new();
        for pattern in &config.files.include {
            if let Ok(glob) = Glob::new(pattern) {
                include_builder.add(glob);
            }
        }
        let include_set = include_builder.build().unwrap_or_else(|_| GlobSet::empty());

        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in &config.files.exclude {
            if let Ok(glob) = Glob::new(pattern) {
                exclude_builder.add(glob);
            }
        }
        let exclude_set = exclude_builder.build().unwrap_or_else(|_| GlobSet::empty());

        Self {
            inner: Arc::new(EngineInner {
                config: Arc::new(config),
                dictionary: Arc::new(dictionary),
                include_set,
                exclude_set,
            }),
        }
    }

    pub fn run(&self, path: PathBuf) -> mpsc::Receiver<SpellError> {
        let scan_root = path.clone();
        let (tx, rx) = mpsc::channel(100);
        let inner = self.inner.clone();

        tokio::spawn(async move {
            let mut walker = WalkBuilder::new(&scan_root);
            walker.add_custom_ignore_filename(".spellcheckignore");
            let walker = walker.build();

            for result in walker {
                match result {
                    Ok(entry) => {
                        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                            let entry_path = entry.path().to_path_buf();
                            let tx = tx.clone();
                            let inner = inner.clone();
                            
                            // Make path relative to scan root for glob matching
                            let relative_path = entry_path.strip_prefix(&scan_root).unwrap_or(&entry_path);
                            
                            if inner.should_check(relative_path) {
                                tokio::spawn(async move {
                                    if let Err(e) = Self::check_file(&entry_path, &inner.dictionary, tx).await {
                                        eprintln!("Error checking {}: {}", entry_path.display(), e);
                                    }
                                });
                            }
                        }
                    }
                    Err(err) => eprintln!("Error: {}", err),
                }
            }
        });

        rx
    }

    async fn check_file(path: &Path, dictionary: &Dictionary, tx: mpsc::Sender<SpellError>) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut disabled = false;

        for (line_num, line_content) in content.lines().enumerate() {
            let line_num = line_num + 1;

            if line_content.contains("spellcheck-disable") {
                disabled = true;
                continue;
            }
            if line_content.contains("spellcheck-enable") {
                disabled = false;
                continue;
            }
            if disabled || line_content.contains("spellcheck-disable-line") {
                continue;
            }

            let words = Self::extract_words(line_content);
            for (col, word) in words {
                if !dictionary.contains(&word) {
                    let _ = tx.send(SpellError {
                        file: path.to_path_buf(),
                        line: line_num,
                        col,
                        word: word.to_string(),
                        context: line_content.to_string(),
                    }).await;
                }
            }
        }

        Ok(())
    }

    fn extract_words(content: &str) -> Vec<(usize, &str)> {
        let mut words = Vec::new();
        let mut start = None;

        for (i, c) in content.char_indices() {
            if c.is_alphabetic() || c == '\'' {
                if start.is_none() {
                    start = Some(i);
                }
            } else {
                if let Some(s) = start {
                    let mut word = &content[s..i];
                    word = word.trim_matches('\'');
                    if word.len() > 1 {
                        words.push((s + 1, word));
                    }
                    start = None;
                }
            }
        }

        if let Some(s) = start {
            let mut word = &content[s..];
            word = word.trim_matches('\'');
            if word.len() > 1 {
                words.push((s + 1, word));
            }
        }

        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_extract_words() {
        let content = "Hello, world! It's a test's line.";
        let words = Engine::extract_words(content);
        let word_list: Vec<&str> = words.into_iter().map(|(_, w)| w).collect();
        assert_eq!(word_list, vec!["Hello", "world", "It's", "test's", "line"]);

        let code = "let y = \"referance\";";
        let words = Engine::extract_words(code);
        let word_list: Vec<&str> = words.into_iter().map(|(_, w)| w).collect();
        assert!(word_list.contains(&"referance"));
    }

    #[test]
    fn test_should_check() {
        let mut config = Config::default();
        config.files.include = vec!["src/**/*.rs".to_string(), "README.md".to_string()];
        config.files.exclude = vec!["**/temp.rs".to_string()];
        
        let dict = Dictionary::new();
        let engine = Engine::new(config, dict);

        assert!(engine.inner.should_check(Path::new("src/main.rs")));
        assert!(engine.inner.should_check(Path::new("./src/main.rs")));
        assert!(engine.inner.should_check(Path::new("README.md")));
        assert!(engine.inner.should_check(Path::new("./README.md")));
        assert!(!engine.inner.should_check(Path::new("docs/index.md")));
        assert!(!engine.inner.should_check(Path::new("src/temp.rs")));
    }
}

impl EngineInner {
    fn should_check(&self, path: &Path) -> bool {
        // Normalize path to forward slashes
        let path_str = path.to_string_lossy().replace('\\', "/");
        
        // If empty or ".", it's the root file being scanned directly
        if path_str.is_empty() || path_str == "." {
            return true;
        }

        // Strip leading "./" if present
        let normalized = path_str.trim_start_matches("./");
        
        if !self.include_set.is_match(normalized) {
            return false;
        }
        if self.exclude_set.is_match(normalized) {
            return false;
        }
        true
    }
}
