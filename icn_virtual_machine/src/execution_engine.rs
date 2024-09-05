// File: icn_virtual_machine/src/execution_engine.rs

/// The Execution Engine is responsible for running smart contracts on the ICN virtual machine.
/// It manages the execution context, handles security validations, and interacts with the state and resources.

use crate::state_manager::StateManager;
use crate::resource_manager::ResourceManager;
use crate::security_manager::SecurityManager;
use std::error::Error;

/// ExecutionEngine struct encapsulates the components needed for executing smart contracts.
pub struct ExecutionEngine {
    pub state_manager: StateManager,
    pub resource_manager: ResourceManager,
    pub security_manager: SecurityManager,
}

/// ExecutionContext holds the current state and data required during the execution of a contract.
pub struct ExecutionContext {
    pub current_key: String,
    pub current_value: String,
}

impl ExecutionContext {
    /// Creates a new instance of ExecutionContext with initial empty values.
    pub fn new() -> Self {
        ExecutionContext {
            current_key: String::new(),
            current_value: String::new(),
        }
    }
}

/// ExecutionResult is used to communicate the result of contract execution back to the caller.
pub enum ExecutionResult {
    Success(String),
    Error(String),
}

impl ExecutionEngine {
    /// Creates a new instance of the ExecutionEngine with default managers.
    pub fn new() -> Self {
        ExecutionEngine {
            state_manager: StateManager::new(),
            resource_manager: ResourceManager::new(),
            security_manager: SecurityManager::new(),
        }
    }

    /// Executes a smart contract by processing its bytecode and updating the state accordingly.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - A reference to a vector of u8 representing the compiled bytecode of the smart contract.
    /// * `context` - A mutable reference to the execution context holding the initial state.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Returns Ok(()) if execution is successful, otherwise returns an error.
    pub fn execute(&mut self, bytecode: &[u8], context: &mut ExecutionContext) -> Result<(), Box<dyn Error>> {
        // Perform security validation on the bytecode before execution.
        if !self.security_manager.validate(bytecode) {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Security validation failed")));
        }

        // Process each bytecode instruction.
        for opcode in bytecode {
            match opcode {
                // Example opcode: 0x01 -> Store value in state
                0x01 => self.state_manager.store_value(context.current_key.clone(), context.current_value.clone()),

                // Example opcode: 0x02 -> Load value from state
                0x02 => context.current_value = self.state_manager.load_value(&context.current_key),

                // Handle unknown opcodes
                _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown opcode"))),
            }
        }

        // Execution successful
        Ok(())
    }
}
