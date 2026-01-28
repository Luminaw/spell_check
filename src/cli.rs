use clap::{Parser, Subcommand};
use crate::config::load_config;
use crate::dictionary::Dictionary;
use crate::engine::Engine;
use std::path::PathBuf;
use colored::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the configuration file.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check files for spelling errors
    Check {
        /// Files or directories to check
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Initialize a new spellcheck.toml file
    Init,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path } => {
            let config_path = if let Some(cfg) = cli.config.as_ref() {
                cfg.clone()
            } else {
                path.join("spellcheck.toml")
            };

            let config = load_config(&config_path)?;
            
            if config_path.exists() {
                println!("{} Using config: {}", "info".blue(), config_path.display());
            } else if cli.config.is_none() {
                // Only show this if they didn't specify a config that doesn't exist
            } else {
                println!("{} Config not found at {:?}, using defaults.", "warn".yellow(), config_path);
            }

            let mut dictionary = Dictionary::new();
            
            // Load embedded dictionary
            let default_words = include_str!("../resources/words.txt");
            for line in default_words.lines() {
                dictionary.add_word(line.trim());
            }

            // Load extra words from config
            dictionary.add_words(&config.dictionary.extra_words);
            
            // Load extra dictionaries
            for dict_path in &config.dictionary.extra_dictionaries {
                dictionary.load_from_file(dict_path)?;
            }

            let engine = Engine::new(config, dictionary);
            let mut rx = engine.run(path);

            let mut count = 0;
            let mut errors = 0;

            while let Some(res) = rx.recv().await {
                match res {
                    Ok(error) => {
                        count += 1;
                        println!(
                            "{} in {}:{}:{}: {}",
                            "Error".red().bold(),
                            error.file.display().to_string().cyan(),
                            error.line.to_string().yellow(),
                            error.col.to_string().yellow(),
                            error.word.bold()
                        );
                        println!("  | {}", error.context.trim());
                        let col = error.col;
                        if col > 0 {
                           println!("  | {:width$}^", "", width = col - 1);
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        eprintln!("{} {}", "error".red().bold(), e);
                    }
                }
            }

            if count == 0 {
                if errors == 0 {
                    println!("{}", "Perfect spelling! No errors found.".green().bold());
                } else {
                    println!("{} Completed with {} processing errors.", "info".blue(), errors);
                }
            } else {
                println!("\nFound {} spelling errors.", count);
                if errors > 0 {
                    println!("(And {} processing errors)", errors);
                }
                std::process::exit(1);
            }
        }
        Commands::Init => {
            let default_config = r#"[files]
include = ["src/**/*.rs", "*.md"]
exclude = ["target/**"]

[dictionary]
extra_words = ["rust", "tokio", "serde"]

[ignore]
words = []
"#;
            std::fs::write("spellcheck.toml", default_config)?;
            println!("Created spellcheck.toml");
        }
    }

    Ok(())
}
