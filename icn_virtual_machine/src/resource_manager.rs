// File: icn_virtual_machine/src/resource_manager.rs

/// The ResourceManager tracks and enforces limits on resources consumed during the execution of smart contracts.
/// This includes gas usage, computational limits, and other resource constraints.

/// ResourceManager struct manages the consumption of resources like gas during contract execution.
pub struct ResourceManager {
    gas_limit: u64,
    gas_used: u64,
}

impl ResourceManager {
    /// Creates a new instance of ResourceManager with a default gas limit.
    pub fn new() -> Self {
        ResourceManager {
            gas_limit: 1_000_000, // Example gas limit
            gas_used: 0,
        }
    }

    /// Consumes a specified amount of gas, returning whether the operation was successful.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of gas to be consumed.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if gas consumption is within the limit, `false` if it exceeds the limit.
    pub fn consume_gas(&mut self, amount: u64) -> bool {
        if self.gas_used + amount > self.gas_limit {
            false
        } else {
            self.gas_used += amount;
            true
        }
    }
}
