// File: icn_core/src/config/config_loader.rs

/// Represents a configuration loader for the ICN node.
pub struct ConfigLoader {
    config: config::Config,
}

impl ConfigLoader {
    /// Creates a new instance of `ConfigLoader` by loading the configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the configuration file.
    ///
    /// # Returns
    ///
    /// * `IcnResult<ConfigLoader>` - An instance of `ConfigLoader` or an `IcnError`.
    pub fn new(config_path: &str) -> IcnResult<Self> {
        let mut config = config::Config::default();
        config
            .merge(config::File::with_name(config_path))
            .map_err(|e| IcnError::Config(format!("Failed to load config: {}", e)))?;

        Ok(ConfigLoader { config })
    }

    /// Retrieves a value from the configuration as a string.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - The configuration value as a string or an `IcnError`.
    pub fn get_value_as_string(&self, key: &str) -> IcnResult<String> {
        self.config
            .get_str(key)
            .map_err(|e| IcnError::Config(format!("Failed to get string value: {}", e)))
    }

    /// Retrieves a value from the configuration as an integer.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<i64>` - The configuration value as an integer or an `IcnError`.
    pub fn get_value_as_int(&self, key: &str) -> IcnResult<i64> {
        self.config
            .get_int(key)
            .map_err(|e| IcnError::Config(format!("Failed to get int value: {}", e)))
    }

    // Add more methods for other types as needed (e.g., boolean, float)
}
