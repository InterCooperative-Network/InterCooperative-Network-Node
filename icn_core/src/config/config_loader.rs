use std::fs;
use serde::Deserialize;
use icn_shared::{IcnError, IcnResult};

/// Represents the application configuration loaded from a TOML file.
///
/// This struct holds configuration details necessary for the server and database
/// components of the application. It is deserialized from a TOML file.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Configuration for the server, such as host, port, and TLS settings.
    pub server: ServerConfig,
    /// Configuration for the database, including connection URLs.
    pub database: DatabaseConfig,
}

/// Configuration for the server, including network and TLS settings.
///
/// This struct is used to configure the server's network-related parameters
/// and TLS (Transport Layer Security) settings for secure communication.
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// The host address where the server will run.
    ///
    /// This can be an IP address or a domain name. Use "0.0.0.0" to bind to all available interfaces.
    pub host: String,

    /// The port on which the server will listen for incoming connections.
    pub port: u16,

    /// Debug mode flag for enabling or disabling verbose output.
    ///
    /// When set to true, additional debug information will be logged.
    pub debug: bool,

    /// The file path to the TLS certificate.
    ///
    /// This should be the full path to the PEM-encoded certificate file.
    pub cert_file_path: String,

    /// The file path to the TLS private key.
    ///
    /// This should be the full path to the PEM-encoded private key file.
    pub key_file_path: String,

    /// The password for the TLS private key, if it is password-protected.
    ///
    /// Leave this empty if the private key is not password-protected.
    pub cert_password: String,
}

/// Configuration for the database, including connection URLs.
///
/// This struct holds the configuration required to connect to one or more databases.
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// A list of database connection URLs.
    pub urls: Vec<String>,
}

/// `ConfigLoader` handles the loading and parsing of TOML configuration files.
///
/// This struct is responsible for reading the configuration file from disk,
/// parsing its content, and providing access to the application's configuration.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// The application's configuration as loaded from the TOML file.
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
    ///
    /// # Errors
    ///
    /// * Returns an `IcnError::Config` if the file cannot be read or parsed.
    pub fn new(config_path: &str) -> IcnResult<Self> {
        // Read the contents of the configuration file
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| IcnError::Config(format!("Failed to read config file '{}': {}", config_path, e)))?;
        
        // Parse the TOML content into a Config struct
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| IcnError::Config(format!("Failed to parse TOML from '{}': {}", config_path, e)))?;
        
        Ok(ConfigLoader { config })
    }

    /// Returns a reference to the loaded configuration.
    ///
    /// # Returns
    ///
    /// * `&Config` - A reference to the configuration.
    ///
    /// # Example
    ///
    /// ```
    /// let config_loader = ConfigLoader::new("config.toml").unwrap();
    /// let config = config_loader.get_config();
    /// println!("Server host: {}", config.server.host);
    /// ```
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// Helper function to create a temporary configuration file for testing.
    fn create_test_config() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
            [server]
            host = "localhost"
            port = 8080
            debug = true
            cert_file_path = "/path/to/cert.pem"
            key_file_path = "/path/to/key.pem"
            cert_password = ""

            [database]
            urls = ["postgresql://user:pass@localhost/db1", "postgresql://user:pass@localhost/db2"]
        "#).unwrap();
        file
    }

    #[test]
    /// Tests the `ConfigLoader` by loading a test configuration and verifying its content.
    fn test_config_loader() {
        let test_config = create_test_config();
        let config_loader = ConfigLoader::new(test_config.path().to_str().unwrap()).unwrap();
        
        let config = config_loader.get_config();

        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.debug);
        assert_eq!(config.server.cert_file_path, "/path/to/cert.pem");
        assert_eq!(config.server.key_file_path, "/path/to/key.pem");
        assert_eq!(config.server.cert_password, "");
        assert_eq!(
            config.database.urls,
            vec![
                "postgresql://user:pass@localhost/db1".to_string(),
                "postgresql://user:pass@localhost/db2".to_string()
            ]
        );
    }
}