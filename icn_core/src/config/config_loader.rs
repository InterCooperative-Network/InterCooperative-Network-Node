// icn_core/src/config/config_loader.rs

use std::fs;
use toml::{Value, map::Map};
use icn_shared::IcnError;

#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config: Map<String, Value>,
}

impl ConfigLoader {
    /// Creates a new ConfigLoader instance by loading a TOML configuration file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// * `Result<Self, IcnError>` - The ConfigLoader instance or an `IcnError` if loading fails
    pub fn new(config_path: &str) -> Result<Self, IcnError> {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| IcnError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Map<String, Value> = toml::from_str(&config_content)
            .map_err(|e| IcnError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(ConfigLoader { config })
    }

    /// Retrieves the entire configuration map.
    ///
    /// # Returns
    ///
    /// * `&Map<String, Value>` - A reference to the configuration map
    pub fn get_config(&self) -> &Map<String, Value> {
        &self.config
    }

    /// Retrieves a string value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the string value to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<String, IcnError>` - The string value or an `IcnError` if not found or invalid
    pub fn get_string(&self, key: &str) -> Result<String, IcnError> {
        self.get_nested_value(key)?
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not a string", key)))
    }

    /// Retrieves an integer value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the integer value to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<i64, IcnError>` - The integer value or an `IcnError` if not found or invalid
    pub fn get_int(&self, key: &str) -> Result<i64, IcnError> {
        self.get_nested_value(key)?
            .as_integer()
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not an integer", key)))
    }

    /// Retrieves a boolean value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the boolean value to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<bool, IcnError>` - The boolean value or an `IcnError` if not found or invalid
    pub fn get_bool(&self, key: &str) -> Result<bool, IcnError> {
        self.get_nested_value(key)?
            .as_bool()
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not a boolean", key)))
    }

    /// Retrieves a nested value from the configuration using a dot-separated key.
    ///
    /// # Arguments
    ///
    /// * `key` - The dot-separated key of the value to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<&Value, IcnError>` - The nested value or an `IcnError` if not found or invalid
    fn get_nested_value(&self, key: &str) -> Result<&Value, IcnError> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.config;

        for (i, &part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                return current.get(part).ok_or_else(|| IcnError::Config(format!("Key '{}' not found", key)));
            }
            current = current.get(part)
                .and_then(|v| v.as_table())
                .ok_or_else(|| IcnError::Config(format!("Invalid nested key: {}", key)))?;
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loader() {
        let config_content = r#"
            [server]
            host = "localhost"
            port = 8080

            [database]
            url = "postgres://user:pass@localhost/dbname"
            max_connections = 100

            [features]
            enable_caching = true
        "#;

        let mut config_file = NamedTempFile::new().unwrap();
        config_file.write_all(config_content.as_bytes()).unwrap();

        let config_loader = ConfigLoader::new(config_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config_loader.get_string("server.host").unwrap(), "localhost");
        assert_eq!(config_loader.get_int("server.port").unwrap(), 8080);
        assert_eq!(config_loader.get_string("database.url").unwrap(), "postgres://user:pass@localhost/dbname");
        assert_eq!(config_loader.get_int("database.max_connections").unwrap(), 100);
        assert_eq!(config_loader.get_bool("features.enable_caching").unwrap(), true);

        assert!(config_loader.get_string("nonexistent.key").is_err());
    }
}
