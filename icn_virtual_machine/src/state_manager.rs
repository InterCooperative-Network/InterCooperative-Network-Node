// File: icn_virtual_machine/src/state_manager.rs

/// The StateManager is responsible for storing and retrieving the state during smart contract execution.
/// It acts as the key-value store where smart contracts can persist data.

use std::collections::HashMap;

/// StateManager struct manages the in-memory state storage for smart contracts.
pub struct StateManager {
    state: HashMap<String, String>,
}

impl StateManager {
    /// Creates a new instance of the StateManager with an empty state.
    pub fn new() -> Self {
        StateManager {
            state: HashMap::new(),
        }
    }

    /// Stores a key-value pair in the state.
    ///
    /// # Arguments
    ///
    /// * `key` - A string key to identify the stored value.
    /// * `value` - The string value to be stored.
    pub fn store_value(&mut self, key: String, value: String) {
        self.state.insert(key, value);
    }

    /// Loads a value from the state using the provided key.
    ///
    /// # Arguments
    ///
    /// * `key` - The string key to retrieve the associated value.
    ///
    /// # Returns
    ///
    /// * `String` - The value associated with the key, or an empty string if the key does not exist.
    pub fn load_value(&self, key: &String) -> String {
        self.state.get(key).cloned().unwrap_or_default()
    }
}
