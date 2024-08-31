use crate::bytecode::Bytecode;

/// The `ExecutionEngine` struct is responsible for interpreting and executing smart contract bytecode.
/// It forms the core logic of the virtual machine within the ICN project.
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Creates a new instance of `ExecutionEngine`.
    ///
    /// # Returns
    ///
    /// * `ExecutionEngine` - A new instance of the `ExecutionEngine` struct.
    pub fn new() -> Self {
        ExecutionEngine
    }

    /// Executes the provided bytecode.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - The bytecode to be executed, represented as a `Bytecode` struct.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Returns `Ok(())` if the execution succeeds, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```
    /// let engine = ExecutionEngine::new();
    /// let bytecode = Bytecode::new(vec![0x01, 0x02]);
    /// let result = engine.execute(bytecode);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// - Returns an error if the bytecode is empty.
    /// - Returns an error if the bytecode contains unknown instructions.
    pub fn execute(&self, bytecode: Bytecode) -> Result<(), String> {
        if bytecode.code.is_empty() {
            return Err("Bytecode is empty".to_string());
        }

        // Example: Interpreting and executing the bytecode.
        // This implementation should be expanded to handle actual instructions.
        for instruction in bytecode.code {
            match instruction {
                0x01 => println!("Instruction: 0x01 - No-op"),
                _ => return Err(format!("Unknown instruction: 0x{:x}", instruction)),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;

    #[test]
    /// Tests the execution of non-empty bytecode.
    fn test_execution_with_non_empty_bytecode() {
        let engine = ExecutionEngine::new();
        let bytecode = Bytecode::new(vec![0x01, 0x02, 0x03]);
        assert!(engine.execute(bytecode).is_ok());
    }

    #[test]
    /// Tests the execution of empty bytecode, expecting an error.
    fn test_execution_with_empty_bytecode() {
        let engine = ExecutionEngine::new();
        let empty_bytecode = Bytecode::new(vec![]);
        assert!(engine.execute(empty_bytecode).is_err());
    }
}
