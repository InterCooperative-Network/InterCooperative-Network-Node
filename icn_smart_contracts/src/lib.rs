// File: icn_smart_contracts/src/lib.rs
// Description: This file defines the SmartContractEngine and SmartContract structures,
// handling operations like deployment and execution of smart contracts.

use std::collections::HashMap;
use icn_virtual_machine::{VirtualMachine, bytecode::Bytecode};
use sha2::{Sha256, Digest};

/// Custom error type for smart contract-related operations
#[derive(Debug, thiserror::Error)]
pub enum SmartContractError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(u32),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Out of gas")]
    OutOfGas,

    #[error("State error: {0}")]
    StateError(String),
}

/// Result type alias for smart contract operations
pub type SmartContractResult<T> = Result<T, SmartContractError>;

/// Represents a smart contract within the ICN ecosystem
#[derive(Debug, Clone)]
pub struct SmartContract {
    pub id: u32,
    pub code: String,
    pub bytecode: Option<Bytecode>,
    pub state: HashMap<String, Vec<u8>>,
}

impl SmartContract {
    /// Create a new instance of a SmartContract
    pub fn new(id: u32, code: &str) -> Self {
        SmartContract {
            id,
            code: code.to_string(),
            bytecode: None,
            state: HashMap::new(),
        }
    }

    /// Set the bytecode for the contract after compilation
    pub fn set_bytecode(&mut self, bytecode: Bytecode) {
        self.bytecode = Some(bytecode);
    }

    /// Update the state of the contract
    pub fn update_state(&mut self, key: &str, value: Vec<u8>) {
        self.state.insert(key.to_string(), value);
    }

    /// Get a value from the contract's state
    pub fn get_state(&self, key: &str) -> Option<&Vec<u8>> {
        self.state.get(key)
    }
}

/// The engine responsible for managing and executing smart contracts
pub struct SmartContractEngine {
    contracts: HashMap<u32, SmartContract>,
    vm: VirtualMachine,
}

impl SmartContractEngine {
    /// Create a new SmartContractEngine
    pub fn new() -> Self {
        SmartContractEngine {
            contracts: HashMap::new(),
            vm: VirtualMachine::new(),
        }
    }

    /// Deploy a smart contract and execute its bytecode on the virtual machine
    pub fn deploy_contract(&mut self, code: &str) -> SmartContractResult<u32> {
        let bytecode = self.compile_contract(code)?;

        let id = self.contracts.len() as u32 + 1;
        let mut contract = SmartContract::new(id, code);
        contract.set_bytecode(Bytecode::new(bytecode.clone()));
        self.contracts.insert(id, contract);

        self.vm.execute(Bytecode::new(bytecode), 1000000) // Set a gas limit for deployment
            .map_err(|e| SmartContractError::ExecutionError(e.to_string()))?;

        Ok(id)
    }

    /// Call a deployed contract by invoking a specific function with arguments
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

    /// Compile the source code of a smart contract into bytecode
    fn compile_contract(&self, code: &str) -> SmartContractResult<Vec<u8>> {
        // Basic compilation logic (this is a simplified example)
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    /// Encode a function call with the provided arguments into bytecode
    fn encode_function_call(&self, function: &str, args: Vec<String>) -> SmartContractResult<Vec<u8>> {
        let mut encoded = function.as_bytes().to_vec();
        for arg in args {
            encoded.extend_from_slice(&[0]); // Argument separator
            encoded.extend_from_slice(arg.as_bytes());
        }
        Ok(encoded)
    }

    /// Retrieve the result from the virtual machine after executing bytecode
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