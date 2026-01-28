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
            let config_path = if let Some(cfg) = cli.config {
                cfg
            } else {
                path.join("spellcheck.toml")
            };

            let config = load_config(&config_path)?;
            
            if config_path.exists() {
                println!("{} Using config: {}", "info".blue(), config_path.display());
            } else {
                println!("{} No config found at {:?}, using defaults.", "info".blue(), config_path);
            }

            let mut dictionary = Dictionary::new();
            
            // Load embedded dictionary
            let default_words = include_str!("../../resources/words.txt");
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
            while let Some(error) = rx.recv().await {
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
                println!("  | {:width$}^", "", width = error.col - 1);
            }

            if count == 0 {
                println!("{}", "Perfect spelling! No errors found.".green().bold());
            } else {
                println!("\nFound {} spelling errors.", count);
                std::process::exit(1);
            }
        }
        Commands::Init => {
            let default_config = "
[files]
include = [\"src/**/*.rs\", \"*.md\"]
exclude = [\"target/**\"]

[dictionary]
extra_words = [\"rust\", \"tokio\", \"serde\"]
";
            std::fs::write("spellcheck.toml", default_config)?;
            println!("Created spellcheck.toml");
        }
    }

    Ok(())
}
