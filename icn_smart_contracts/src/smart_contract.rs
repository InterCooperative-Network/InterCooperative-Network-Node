// File: icn_smart_contracts/src/smart_contract.rs

/// The SmartContract module defines the structure and deployment mechanisms for smart contracts.
/// It integrates with the ICN Virtual Machine to enable contract execution on the network.

use icn_virtual_machine::execution_engine::{ExecutionEngine, ExecutionContext, ExecutionResult};

/// SmartContract struct represents a compiled smart contract ready for deployment on the ICN.
pub struct SmartContract {
    pub bytecode: Vec<u8>,
}

impl SmartContract {
    /// Creates a new instance of SmartContract with the provided bytecode.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - A vector of u8 representing the compiled bytecode of the smart contract.
    pub fn new(bytecode: Vec<u8>) -> Self {
        SmartContract { bytecode }
    }

    /// Deploys the smart contract on the provided execution engine.
    ///
    /// # Arguments
    ///
    /// * `vm` - A mutable reference to the ExecutionEngine instance.
    /// * `context` - The execution context for the contract deployment.
    ///
    /// # Returns
    ///
    /// * `ExecutionResult` - The result of the contract deployment.
    pub fn deploy(&self, vm: &mut ExecutionEngine, context: ExecutionContext) -> ExecutionResult {
        vm.execute_contract(self.bytecode.clone(), context)
    }
}
