use crate::config_schema::Config;
use crate::dictionary::Dictionary;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::task::JoinSet;

pub struct Engine {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    config: Arc<Config>,
    dictionary: Arc<Dictionary>,
    include_set: GlobSet,
    exclude_set: GlobSet,
}

#[derive(Debug, Clone)]
pub struct SpellError {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub word: String,
    pub context: String,
}

impl Engine {
    pub fn try_new(config: Config, dictionary: Dictionary) -> Result<Self> {
        let mut include_builder = GlobSetBuilder::new();
        for pattern in &config.files.include {
            let glob = Glob::new(pattern).with_context(|| format!("Invalid include glob pattern: {}", pattern))?;
            include_builder.add(glob);
        }
        let include_set = include_builder.build().context("Failed to build include glob set")?;

        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in &config.files.exclude {
            let glob = Glob::new(pattern).with_context(|| format!("Invalid exclude glob pattern: {}", pattern))?;
            exclude_builder.add(glob);
        }
        let exclude_set = exclude_builder.build().context("Failed to build exclude glob set")?;

        Ok(Self {
            inner: Arc::new(EngineInner {
                config: Arc::new(config),
                dictionary: Arc::new(dictionary),
                include_set,
                exclude_set,
            }),
        })
    }

    pub fn run(&self, path: PathBuf) -> mpsc::Receiver<Result<SpellError, String>> {
        let (tx, rx) = mpsc::channel(100);
        let inner = self.inner.clone();
        let scan_root = path.clone();

        tokio::spawn(async move {
            let mut walker = WalkBuilder::new(&scan_root);
            walker.add_custom_ignore_filename(".spellcheckignore");
            let walker = walker.build();

            let mut set = JoinSet::new();
            
            for result in walker {
                match result {
                    Ok(entry) => {
                        if entry.file_type().map(|ft| ft.is_file()).expect("Failed to get file type") {
                            let entry_path = entry.path().to_path_buf();
                            
                            // Make path relative to scan root for glob matching
                            let relative_path = entry_path.strip_prefix(&scan_root).expect("Failed to get relative path");
                            
                            if inner.should_check(relative_path) {
                                let tx = tx.clone();
                                let inner = inner.clone();
                                let entry_path = entry_path.clone();
                                
                                // Limit concurrency by checking how many tasks are active
                                if set.len() >= 20 {
                                    set.join_next().await;
                                }
                                
                                set.spawn(async move {
                                    if let Err(e) = Self::check_file(&entry_path, &inner, tx).await {
                                        // Errors are handled inside check_file or reported back if critical
                                        return Err(format!("Error checking {}: {}", entry_path.display(), e));
                                    }
                                    Ok(())
                                });
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(format!("Walk error: {}", err))).await;
                    }
                }
            }

            while let Some(res) = set.join_next().await {
                if let Ok(Err(e)) = res {
                    let _ = tx.send(Err(e)).await;
                }
            }
        });

        rx
    }

    async fn check_file(path: &Path, inner: &EngineInner, tx: mpsc::Sender<Result<SpellError, String>>) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file {}", path.display()))?;
        
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
                if !inner.dictionary.contains(word) {
                    // Check if word is in ignore list
                    if inner.config.ignore.words.iter().any(|w| w.eq_ignore_ascii_case(word)) {
                        continue;
                    }

                    let _ = tx.send(Ok(SpellError {
                        file: path.to_path_buf(),
                        line: line_num,
                        col,
                        word: word.to_string(),
                        context: line_content.to_string(),
                    })).await;
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
                    if word.len() > 1 && !word.chars().any(char::is_numeric) {
                        words.push((s + 1, word));
                    }
                    start = None;
                }
            }
        }

        if let Some(s) = start {
            let mut word = &content[s..];
            word = word.trim_matches('\'');
            if word.len() > 1 && !word.chars().any(char::is_numeric) {
                words.push((s + 1, word));
            }
        }

        words
    }
}

impl EngineInner {
    fn should_check(&self, path: &Path) -> bool {
        // Normalize path to forward slashes for globset
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_schema::Config;

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
    fn test_should_check() -> anyhow::Result<()> {
        let mut config = Config::default();
        config.files.include = vec!["src/**/*.rs".to_string(), "README.md".to_string()];
        config.files.exclude = vec!["**/temp.rs".to_string()];
        
        let dict = Dictionary::new();
        let engine = Engine::try_new(config, dict)?;

        assert!(engine.inner.should_check(Path::new("src/main.rs")));
        assert!(engine.inner.should_check(Path::new("./src/main.rs")));
        assert!(engine.inner.should_check(Path::new("README.md")));
        assert!(engine.inner.should_check(Path::new("./README.md")));
        assert!(!engine.inner.should_check(Path::new("docs/index.md")));
        assert!(!engine.inner.should_check(Path::new("src/temp.rs")));
        Ok(())
    }
}
