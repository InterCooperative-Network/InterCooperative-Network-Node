// File: icn_core/src/config/config_loader.rs
// This file defines the `ConfigLoader` struct responsible for loading and parsing TOML configuration files.
// It also defines the `Config` struct representing the application's configuration.

use std::fs;
use serde::Deserialize;
use icn_shared::{IcnError, IcnResult};

/// Represents the application configuration loaded from a TOML file.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub urls: Vec<String>,
}

/// `ConfigLoader` handles the loading and parsing of TOML configuration files.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config: Config,
}

impl ConfigLoader {
    /// Creates a new `ConfigLoader` instance by loading and parsing a TOML configuration file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the TOML configuration file.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Self>` - A new `ConfigLoader` instance if successful, otherwise an `IcnError`.
    pub fn new(config_path: &str) -> IcnResult<Self> {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| IcnError::Config(format!("Failed to read config file '{}': {}", config_path, e)))?;
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| IcnError::Config(format!("Failed to parse TOML from '{}': {}", config_path, e)))?;
        Ok(ConfigLoader { config })
    }

    /// Returns a reference to the loaded configuration.
    ///
    /// # Returns
    ///
    /// * `&Config` - A reference to the configuration.
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_config() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
            [server]
            host = "localhost"
            port = 8080
            debug = true

            [database]
            urls = ["postgresql://user:pass@localhost/db1", "postgresql://user:pass@localhost/db2"]
        "#).unwrap();
        file
    }

    #[test]
    fn test_config_loader() {
        let config_file = create_test_config();
        let config_loader = ConfigLoader::new(config_file.path().to_str().unwrap()).unwrap();

        let config = config_loader.get_config();

        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.debug);
        assert_eq!(
            config.database.urls,
            vec![
                "postgresql://user:pass@localhost/db1".to_string(),
                "postgresql://user:pass@localhost/db2".to_string()
            ]
        );
    }
}
