// File: icn_virtual_machine/src/plugin_manager.rs

use std::fs;

/// The PluginManager allows extending the VM's capabilities by loading external plugins.
pub struct PluginManager {
    /// A vector to keep track of loaded plugins (plugin names or paths).
    loaded_plugins: Vec<String>,
}

impl PluginManager {
    /// Creates a new instance of PluginManager.
    ///
    /// # Returns
    /// * `PluginManager` - A new instance of PluginManager with an empty plugin list.
    pub fn new() -> Self {
        PluginManager {
            loaded_plugins: Vec::new(),
        }
    }

    /// Loads a plugin from the given path and registers it with the VM.
    ///
    /// # Arguments
    /// * `plugin_path` - The file path to the plugin that needs to be loaded.
    ///
    /// # Returns
    /// * `Result<String, String>` - Returns the plugin path if loaded successfully,
    ///   or an error string if loading failed.
    pub fn load_plugin(&mut self, plugin_path: &str) -> Result<String, String> {
        // Check if the plugin path is valid and exists
        if plugin_path.is_empty() {
            return Err("Plugin path cannot be empty".to_string());
        }

        // Validate if the file exists at the given path
        if !fs::metadata(plugin_path).is_ok() {
            return Err(format!("Plugin not found at path: {}", plugin_path));
        }

        // Add the plugin to the list of loaded plugins
        self.loaded_plugins.push(plugin_path.to_string());

        // Return success message with the plugin path
        Ok(format!("Plugin loaded successfully from: {}", plugin_path))
    }

    /// Unloads a plugin from the manager.
    ///
    /// # Arguments
    /// * `plugin_path` - The file path to the plugin to be unloaded.
    ///
    /// # Returns
    /// * `Result<String, String>` - Returns the unloaded plugin path if successful,
    ///   or an error string if the plugin was not found.
    pub fn unload_plugin(&mut self, plugin_path: &str) -> Result<String, String> {
        // Check if the plugin exists in the loaded list
        if let Some(pos) = self.loaded_plugins.iter().position(|p| p == plugin_path) {
            self.loaded_plugins.remove(pos);
            return Ok(format!("Plugin unloaded successfully: {}", plugin_path));
        }

        // If plugin not found, return an error
        Err(format!("Plugin not found: {}", plugin_path))
    }

    /// Returns a list of currently loaded plugins.
    ///
    /// # Returns
    /// * `Vec<String>` - A vector containing the paths or identifiers of the loaded plugins.
    pub fn get_loaded_plugins(&self) -> Vec<String> {
        self.loaded_plugins.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        
        // Test loading a plugin (simulate a valid path for testing)
        let plugin_path = "/path/to/plugin";
        let result = manager.load_plugin(plugin_path);
        assert!(result.is_ok());
        assert_eq!(manager.get_loaded_plugins().len(), 1);

        // Test unloading the plugin
        let unload_result = manager.unload_plugin(plugin_path);
        assert!(unload_result.is_ok());
        assert!(manager.get_loaded_plugins().is_empty());

        // Test loading an invalid plugin path
        let invalid_path = "";
        let invalid_result = manager.load_plugin(invalid_path);
        assert!(invalid_result.is_err());
    }
}
