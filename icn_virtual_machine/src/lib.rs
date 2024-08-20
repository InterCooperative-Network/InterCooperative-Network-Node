// icn_virtual_machine/src/lib.rs
pub mod bytecode;
pub mod execution_engine;

use bytecode::Bytecode;
use execution_engine::ExecutionEngine;

/// The `VirtualMachine` struct is responsible for executing smart contracts.
/// It processes bytecode using an execution engine.
pub struct VirtualMachine {
    engine: ExecutionEngine,
}

impl VirtualMachine {
    /// Creates a new instance of the `VirtualMachine`.
    pub fn new() -> Self {
        VirtualMachine {
            engine: ExecutionEngine::new(),
        }
    }

    /// Executes a smart contract represented by bytecode.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - The bytecode to be executed.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - `Ok` if execution succeeds, otherwise an error message.
    pub fn execute(&mut self, bytecode: Bytecode) -> Result<(), String> {
        self.engine.execute(bytecode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_machine_execution() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0x01, 0x02, 0x03]);

        // Test execution with non-empty bytecode
        assert!(vm.execute(bytecode).is_ok());

        // Test execution with empty bytecode
        let empty_bytecode = Bytecode::new(vec![]);
        assert!(vm.execute(empty_bytecode).is_err());
    }
}
