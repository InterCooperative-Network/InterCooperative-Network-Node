// File: icn_virtual_machine/src/execution_engine.rs

/// The Execution Engine is responsible for running smart contracts on the ICN virtual machine.
/// It manages the execution context, handles security validations, and interacts with the state and resources.

use crate::state_manager::StateManager;
use crate::resource_manager::ResourceManager;
use crate::security_manager::SecurityManager;

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
    /// * `bytecode` - A vector of u8 representing the compiled bytecode of the smart contract.
    /// * `context` - The execution context holding the initial state.
    ///
    /// # Returns
    ///
    /// * `ExecutionResult` - The outcome of the contract execution, either success or error.
    pub fn execute_contract(&mut self, bytecode: Vec<u8>, context: ExecutionContext) -> ExecutionResult {
        // Perform security validation on the bytecode before execution.
        if !self.security_manager.validate(&bytecode) {
            return ExecutionResult::Error("Security validation failed".to_string());
        }

        // Initialize the execution context.
        let mut ctx = context;

        // Process each bytecode instruction.
        for opcode in bytecode {
            match opcode {
                // Example opcode: 0x01 -> Store value in state
                0x01 => self.state_manager.store_value(ctx.current_key.clone(), ctx.current_value.clone()),
                // Example opcode: 0x02 -> Load value from state
                0x02 => ctx.current_value = self.state_manager.load_value(&ctx.current_key),
                // Handle unknown opcodes
                _ => return ExecutionResult::Error("Unknown opcode".to_string()),
            }
        }

        // Return the result of the execution.
        ExecutionResult::Success(ctx.current_value)
    }
}
