use std::collections::HashSet;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

pub struct Dictionary {
    words: HashSet<String>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            words: HashSet::new(),
        }
    }

    pub fn add_word(&mut self, word: &str) {
        self.words.insert(word.to_lowercase());
    }

    pub fn add_words<I, S>(&mut self, words: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for word in words {
            self.add_word(word.as_ref());
        }
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read dictionary file at {:?}", path))?;

        for line in content.lines() {
            let word = line.trim();
            if !word.is_empty() {
                self.add_word(word);
            }
        }
        Ok(())
    }

    pub fn contains(&self, word: &str) -> bool {
        let word_lower = word.to_lowercase();
        // Check exact match
        if self.words.contains(&word_lower) {
            return true;
        }

        // Handle common suffixes or variations if needed, 
        // but for a basic spellchecker, exact match in lowercase is the baseline.
        false
    }

    pub fn count(&self) -> usize {
        self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_lookup() {
        let mut dict = Dictionary::new();
        dict.add_word("Rust");
        dict.add_word("tokio");

        assert!(dict.contains("rust"));
        assert!(dict.contains("RUST"));
        assert!(dict.contains("tokio"));
        assert!(!dict.contains("missing"));
    }

    #[test]
    fn test_add_words() {
        let mut dict = Dictionary::new();
        dict.add_words(vec!["one", "two", "three"]);
        assert_eq!(dict.count(), 3);
        assert!(dict.contains("TWO"));
    }
}
