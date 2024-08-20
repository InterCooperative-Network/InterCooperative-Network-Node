// icn_storage/src/state_storage.rs

use std::collections::HashMap;

/// The `StateStorage` struct is responsible for managing the state of the blockchain.
/// It uses an in-memory `HashMap` to store key-value pairs representing the state.
pub struct StateStorage {
    storage: HashMap<String, String>,
}

impl StateStorage {
    /// Creates a new `StateStorage` instance.
    pub fn new() -> Self {
        StateStorage {
            storage: HashMap::new(),
        }
    }

    /// Updates the state storage with a new key-value pair.
    pub fn update_state(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.storage.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Retrieves a value from the state storage by its key.
    pub fn get_state(&self, key: &str) -> Option<String> {
        self.storage.get(key).cloned()
    }
}
