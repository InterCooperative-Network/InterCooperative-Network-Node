//! # ICN Smart Contracts Module
//!
//! This module defines the core structures and functionality for managing smart contracts
//! within the InterCooperative Network (ICN) ecosystem. It provides a robust framework for
//! deploying, executing, and managing smart contracts using a simplified virtual machine.
//!
//! ## Key Components
//!
//! - `SmartContract`: Represents an individual smart contract with its associated data.
//! - `SmartContractEngine`: Manages the lifecycle of smart contracts, including deployment,
//!   execution, and state management.
//! - `SmartContractError`: Custom error types for precise error handling in contract operations.

use std::collections::HashMap;
use icn_virtual_machine::{VirtualMachine, bytecode::Bytecode};
use sha2::{Sha256, Digest};

/// Custom error type for smart contract-related operations.
///
/// This enum provides a comprehensive set of error variants to handle various
/// failure modes in smart contract operations, enabling precise error reporting
/// and handling throughout the smart contract lifecycle.
#[derive(Debug, thiserror::Error)]
pub enum SmartContractError {
    /// Error for invalid arguments passed to the contract
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Error when a contract is not found by ID
    #[error("Contract not found: {0}")]
    ContractNotFound(u32),

    /// Error encountered during contract compilation
    #[error("Compilation error: {0}")]
    CompilationError(String),

    /// Error encountered during contract execution in the VM
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Error for unsupported operations in the VM or contract
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Error when a contract runs out of gas
    #[error("Out of gas")]
    OutOfGas,

    /// Error related to contract state manipulation
    #[error("State error: {0}")]
    StateError(String),
}

/// Result type alias for operations in the smart contract engine.
///
/// This type alias simplifies the return types of functions that may produce
/// a `SmartContractError`, improving code readability and consistency.
pub type SmartContractResult<T> = Result<T, SmartContractError>;

/// Represents a smart contract within the ICN ecosystem.
///
/// A `SmartContract` encapsulates all the necessary information for a deployable
/// and executable contract, including its source code, compiled bytecode, and
/// current state.
#[derive(Debug, Clone)]
pub struct SmartContract {
    /// Unique identifier for the contract
    pub id: u32,
    /// Source code of the contract
    pub code: String,
    /// Optional compiled bytecode of the contract
    pub bytecode: Option<Bytecode>,
    /// Contract's internal state, mapping keys to byte arrays
    pub state: HashMap<String, Vec<u8>>,
}

impl SmartContract {
    /// Creates a new instance of a SmartContract.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique ID for the contract
    /// * `code` - The source code of the contract
    ///
    /// # Returns
    ///
    /// A new instance of `SmartContract` with initialized fields.
    pub fn new(id: u32, code: &str) -> Self {
        SmartContract {
            id,
            code: code.to_string(),
            bytecode: None,
            state: HashMap::new(),
        }
    }

    /// Sets the bytecode for the contract after successful compilation.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - Compiled bytecode for the contract
    pub fn set_bytecode(&mut self, bytecode: Bytecode) {
        self.bytecode = Some(bytecode);
    }

    /// Updates the state of the contract with a given key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - Key for the state entry
    /// * `value` - Value to be stored in the contract's state
    pub fn update_state(&mut self, key: &str, value: Vec<u8>) {
        self.state.insert(key.to_string(), value);
    }

    /// Retrieves a value from the contract's state.
    ///
    /// # Arguments
    ///
    /// * `key` - Key for the state entry to retrieve
    ///
    /// # Returns
    ///
    /// An optional reference to the value if it exists in the state.
    pub fn get_state(&self, key: &str) -> Option<&Vec<u8>> {
        self.state.get(key)
    }
}

/// The engine responsible for managing and executing smart contracts.
///
/// `SmartContractEngine` handles the deployment, execution, and state management
/// of smart contracts. It interacts with the virtual machine to execute contract
/// bytecode and manages gas consumption.
pub struct SmartContractEngine {
    /// Map of deployed contracts by their unique IDs
    contracts: HashMap<u32, SmartContract>,
    /// Virtual machine instance for executing contract bytecode
    vm: VirtualMachine,
}

impl SmartContractEngine {
    /// Creates a new instance of the SmartContractEngine.
    ///
    /// # Returns
    ///
    /// A new `SmartContractEngine` with an empty contract map and initialized VM.
    pub fn new() -> Self {
        SmartContractEngine {
            contracts: HashMap::new(),
            vm: VirtualMachine::new(),
        }
    }

    /// Deploys a smart contract and executes its bytecode on the virtual machine.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code of the contract to be deployed
    ///
    /// # Returns
    ///
    /// The unique contract ID if successful, or an error if deployment fails.
    pub fn deploy_contract(&mut self, code: &str) -> SmartContractResult<u32> {
        let bytecode = self.compile_contract(code)?;

        let id = self.contracts.len() as u32 + 1;
        let mut contract = SmartContract::new(id, code);
        contract.set_bytecode(Bytecode::new(bytecode.clone()));
        self.contracts.insert(id, contract);

        // Execute the contract's bytecode in the VM with a gas limit
        self.vm.execute(Bytecode::new(bytecode), 1000000)
            .map_err(|e| SmartContractError::ExecutionError(e.to_string()))?;

        Ok(id)
    }

    /// Calls a deployed contract by invoking a function with arguments.
    ///
    /// # Arguments
    ///
    /// * `id` - The contract ID to call
    /// * `function` - The function to invoke in the contract
    /// * `args` - Arguments to pass to the function
    ///
    /// # Returns
    ///
    /// The result of the function call, or an error if the call fails.
    pub fn call_contract(&mut self, id: u32, function: &str, args: Vec<String>) -> SmartContractResult<String> {
        let contract = self.contracts.get_mut(&id)
            .ok_or_else(|| SmartContractError::ContractNotFound(id))?;

        let call_data = self.encode_function_call(function, args)?;

        let bytecode = contract.bytecode.clone()
            .ok_or_else(|| SmartContractError::ExecutionError("Contract bytecode not available".to_string()))?;
        
        let (result, gas_used) = self.vm.execute_with_state(bytecode, call_data, &mut contract.state, 1000000)
            .map_err(|e| SmartContractError::ExecutionError(e.to_string()))?;

        println!("Gas used: {}", gas_used);

        self.get_vm_result(result)
    }

    /// Compiles the source code of a smart contract into bytecode.
    ///
    /// This method uses SHA-256 hashing as a simplified compilation mechanism.
    /// In a production environment, this would be replaced with a proper compiler.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to compile
    ///
    /// # Returns
    ///
    /// The compiled bytecode, or an error if compilation fails.
    fn compile_contract(&self, code: &str) -> SmartContractResult<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    /// Encodes a function call with arguments into bytecode.
    ///
    /// # Arguments
    ///
    /// * `function` - The function name to call
    /// * `args` - The arguments to encode
    ///
    /// # Returns
    ///
    /// Encoded bytecode representing the function call and arguments.
    fn encode_function_call(&self, function: &str, args: Vec<String>) -> SmartContractResult<Vec<u8>> {
        let mut encoded = function.as_bytes().to_vec();
        for arg in args {
            encoded.extend_from_slice(&[0]); // Simple argument separator
            encoded.extend_from_slice(arg.as_bytes());
        }
        Ok(encoded)
    }

    /// Retrieves and processes the result from the virtual machine.
    ///
    /// # Arguments
    ///
    /// * `result` - The raw result from the VM as a byte vector
    ///
    /// # Returns
    ///
    /// The processed result as a UTF-8 string, or an error if parsing fails.
    fn get_vm_result(&self, result: Vec<u8>) -> SmartContractResult<String> {
        String::from_utf8(result)
            .map_err(|e| SmartContractError::ExecutionError(format!("Failed to parse VM result: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_contract() {
        let mut engine = SmartContractEngine::new();
        let code = "contract Test { function greet() public pure returns (string memory) { return \"Hello, World!\"; } }";
        let result = engine.deploy_contract(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_call_contract() {
        let mut engine = SmartContractEngine::new();
        let code = "contract Test { function greet() public pure returns (string memory) { return \"Hello, World!\"; } }";
        let contract_id = engine.deploy_contract(code).unwrap();
        let result = engine.call_contract(contract_id, "greet", vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_contract_not_found() {
        let mut engine = SmartContractEngine::new();
        let result = engine.call_contract(1, "greet", vec![]);
        assert!(matches!(result, Err(SmartContractError::ContractNotFound(1))));
    }

    #[test]
    fn test_contract_state() {
        let mut engine = SmartContractEngine::new();
        let code = "contract Test { uint256 public value; function setValue(uint256 _value) public { value = _value; } }";
        let contract_id = engine.deploy_contract(code).unwrap();
        
        // Set value
        let result = engine.call_contract(contract_id, "setValue", vec!["42".to_string()]);
        assert!(result.is_ok());

        // Get value
        let result = engine.call_contract(contract_id, "value", vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_out_of_gas() {
        let mut engine = SmartContractEngine::new();
        let code = "contract Test { function infiniteLoop() public { while(true) {} } }";
        let contract_id = engine.deploy_contract(code).unwrap();
        let result = engine.call_contract(contract_id, "infiniteLoop", vec![]);
        assert!(matches!(result, Err(SmartContractError::ExecutionError(_))));
    }
}