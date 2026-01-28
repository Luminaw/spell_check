//! # Spell Check
//!
//! `spell_check` is a high-performance, memory-safe spell-checking tool designed for developers.
//! It supports concurrent scanning, custom dictionaries, and robust configuration.

pub mod config;
pub mod config_schema;
pub mod engine;
pub mod cli;
pub mod dictionary;
