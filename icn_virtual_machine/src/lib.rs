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
    pub fn execute(&mut self, bytecode: Bytecode) -> Result<(), String> {
        self.engine.execute(bytecode)
    }
}
