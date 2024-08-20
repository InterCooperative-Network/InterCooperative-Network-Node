// icn_core/src/config/config_loader.rs

use std::fs;
use toml::{Value, map::Map};
use icn_shared::IcnResult;

/// `ConfigLoader` handles the loading and parsing of TOML configuration files.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config: Map<String, Value>,
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
        let config_content = fs::read_to_string(config_path)?;
        let config: Map<String, Value> = toml::from_str(&config_content)?;
        Ok(ConfigLoader { config })
    }

    /// Retrieves the entire configuration map.
    ///
    /// # Returns
    ///
    /// * `&Map<String, Value>` - A reference to the configuration map.
    pub fn get_config(&self) -> &Map<String, Value> {
        &self.config
    }

    /// Retrieves a string value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the string value to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - The string value or an `IcnError` if not found or invalid.
    pub fn get_string(&self, key: &str) -> IcnResult<String> {
        self.get_nested_value(key)?
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not a string", key)))
    }

    /// Retrieves an integer value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the integer value to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<i64>` - The integer value or an `IcnError` if not found or invalid.
    pub fn get_int(&self, key: &str) -> IcnResult<i64> {
        self.get_nested_value(key)?
            .as_integer()
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not an integer", key)))
    }

    /// Retrieves a boolean value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the boolean value to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - The boolean value or an `IcnError` if not found or invalid.
    pub fn get_bool(&self, key: &str) -> IcnResult<bool> {
        self.get_nested_value(key)?
            .as_bool()
            .ok_or_else(|| IcnError::Config(format!("Value for key '{}' is not a boolean", key)))
    }

    /// Retrieves a nested value from the configuration using dot-separated keys.
    ///
    /// # Arguments
    ///
    /// * `key` - The dot-separated keys to the nested value.
    ///
    /// # Returns
    ///
    /// * `IcnResult<&Value>` - A reference to the nested value or an `IcnError` if not found.
    fn get_nested_value(&self, key: &str) -> IcnResult<&Value> {
        let keys: Vec<&str> = key.split('.').collect();
        let mut current_value = &self.config;

        for key_part in keys {
            current_value = current_value
                .get(key_part)
                .ok_or_else(|| IcnError::Config(format!("Key '{}' not found", key)))?;
        }

        Ok(current_value)
    }
}