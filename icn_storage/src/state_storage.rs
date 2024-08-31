// file: icn_storage/src/state_storage.rs

use std::collections::HashMap;
use icn_shared::IcnResult;

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
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    /// * `value` - A string slice that holds the value.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the state is successfully updated.
    pub fn update_state(&mut self, key: &str, value: &str) -> IcnResult<()> {
        self.storage.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Retrieves a value from the state storage by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - Returns an `Option` containing the value associated with the key,
    ///   or `None` if the key does not exist.
    pub fn get_state(&self, key: &str) -> Option<String> {
        self.storage.get(key).cloned()
    }

    /// Removes a key-value pair from the state storage.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to be removed.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the key-value pair is successfully removed or didn't exist.
    pub fn remove_state(&mut self, key: &str) -> IcnResult<()> {
        self.storage.remove(key);
        Ok(())
    }

    /// Checks if a key exists in the state storage.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to check.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the key exists, `false` otherwise.
    pub fn has_state(&self, key: &str) -> bool {
        self.storage.contains_key(key)
    }

    /// Returns the number of key-value pairs in the state storage.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of key-value pairs in the storage.
    pub fn state_count(&self) -> usize {
        self.storage.len()
    }

    /// Clears all key-value pairs from the state storage.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the storage is successfully cleared.
    pub fn clear_state(&mut self) -> IcnResult<()> {
        self.storage.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_and_get_state() {
        let mut storage = StateStorage::new();
        assert!(storage.update_state("key1", "value1").is_ok());
        assert_eq!(storage.get_state("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_remove_state() {
        let mut storage = StateStorage::new();
        storage.update_state("key1", "value1").unwrap();
        assert!(storage.remove_state("key1").is_ok());
        assert_eq!(storage.get_state("key1"), None);
    }

    #[test]
    fn test_has_state_and_state_count() {
        let mut storage = StateStorage::new();
        storage.update_state("key1", "value1").unwrap();
        storage.update_state("key2", "value2").unwrap();
        assert!(storage.has_state("key1"));
        assert!(!storage.has_state("key3"));
        assert_eq!(storage.state_count(), 2);
    }

    #[test]
    fn test_clear_state() {
        let mut storage = StateStorage::new();
        storage.update_state("key1", "value1").unwrap();
        storage.update_state("key2", "value2").unwrap();
        assert!(storage.clear_state().is_ok());
        assert_eq!(storage.state_count(), 0);
    }
}