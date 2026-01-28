//! Dictionary implementation for fast word lookups.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// A thread-safe, case-insensitive dictionary used for word lookups.
pub struct Dictionary {
    words: HashSet<String>,
}

impl Dictionary {
    /// Creates a new, empty `Dictionary`.
    pub fn new() -> Self {
        Self {
            words: HashSet::new(),
        }
    }

    /// Adds a single word to the dictionary.
    ///
    /// The word is normalized to lowercase before storage.
    pub fn add_word(&mut self, word: &str) {
        self.words.insert(word.to_lowercase());
    }

    /// Adds multiple words to the dictionary from an iterator.
    pub fn add_words<I, S>(&mut self, words: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for word in words {
            self.add_word(word.as_ref());
        }
    }

    /// Loads words from a plain-text file, one word per line.
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

    /// Checks if a word exists in the dictionary.
    ///
    /// This lookup is case-insensitive.
    pub fn contains(&self, word: &str) -> bool {
        let word_lower = word.to_lowercase();
        self.words.contains(&word_lower)
    }

    /// Returns the total number of words in the dictionary.
    pub fn count(&self) -> usize {
        self.words.len()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
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
