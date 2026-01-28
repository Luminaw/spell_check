use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use crate::config_schema::Config;

pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| "Failed to parse TOML configuration")?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "[dictionary]\nextra_words = [\"custom\"]")?;
        
        let config = load_config(file.path())?;
        assert_eq!(config.dictionary.extra_words, vec!["custom"]);
        Ok(())
    }

    #[test]
    fn test_default_config() -> Result<()> {
        let config = Config::default();
        assert!(config.files.include.contains(&"**/*.{md,txt,rs,js,ts,py,c,cpp,h,hpp,go,java}".to_string()));
        Ok(())
    }
}
