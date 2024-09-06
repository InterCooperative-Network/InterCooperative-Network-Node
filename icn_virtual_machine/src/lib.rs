// File: icn_virtual_machine/src/lib.rs

use std::collections::HashMap;
pub mod bytecode;
use self::bytecode::Bytecode;
use icn_shared::{IcnError, IcnResult};

/// Represents the Virtual Machine for executing smart contracts
pub struct VirtualMachine {
    /// Memory space for the virtual machine (64KB)
    memory: Vec<u8>,
    /// Stack for operation execution
    stack: Vec<i64>,
    /// Current instruction pointer
    program_counter: usize,
    /// Remaining gas for execution
    gas_remaining: u64,
}

impl VirtualMachine {
    /// Creates a new instance of the Virtual Machine
    ///
    /// # Returns
    ///
    /// * `Self` - A new VirtualMachine instance with initialized memory and empty stack
    pub fn new() -> Self {
        VirtualMachine {
            memory: vec![0; 65536], // 64KB of memory
            stack: Vec::new(),
            program_counter: 0,
            gas_remaining: 0,
        }
    }

    /// Executes the given bytecode
    ///
    /// # Arguments
    ///
    /// * `bytecode` - The bytecode to execute
    /// * `gas_limit` - The maximum amount of gas that can be used for execution
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if execution was successful, Err otherwise
    pub fn execute(&mut self, bytecode: Bytecode, gas_limit: u64) -> IcnResult<()> {
        self.gas_remaining = gas_limit;
        self.program_counter = 0;
        self.stack.clear();

        while self.program_counter < bytecode.code.len() {
            if self.gas_remaining == 0 {
                return Err(IcnError::a("Execution halted: Out of gas".to_string()));
            }

            let opcode = bytecode.code[self.program_counter];
            self.program_counter += 1;

            match opcode {
                0x01 => self.op_add()?,
                0x02 => self.op_sub()?,
                0x03 => self.op_mul()?,
                0x04 => self.op_div()?,
                0x10 => self.op_push()?,
                0x11 => self.op_pop()?,
                0x20 => self.op_jump()?,
                0x21 => self.op_jumpi()?,
                0xFF => break, // HALT
                _ => return Err(IcnError::VirtualMachine(format!("Execution error: Invalid opcode 0x{:02X}", opcode))),
            }
        }

        Ok(())
    }

    /// Executes the given bytecode with a specific state
    ///
    /// # Arguments
    ///
    /// * `bytecode` - The bytecode to execute
    /// * `call_data` - Input data for the execution
    /// * `state` - The current state of the contract
    /// * `gas_limit` - The maximum amount of gas that can be used for execution
    ///
    /// # Returns
    ///
    /// * `IcnResult<(Vec<u8>, u64)>` - The execution result and gas used, or an error
    pub fn execute_with_state(
        &mut self,
        bytecode: Bytecode,
        call_data: Vec<u8>,
        state: &mut HashMap<String, Vec<u8>>,
        gas_limit: u64,
    ) -> IcnResult<(Vec<u8>, u64)> {
        self.gas_remaining = gas_limit;
        self.program_counter = 0;
        self.stack.clear();

        // Load call data into memory
        for (i, &byte) in call_data.iter().enumerate() {
            if i >= self.memory.len() {
                return Err(IcnError::VirtualMachine("Memory access error: Call data exceeds memory size".to_string()));
            }
            self.memory[i] = byte;
        }

        self.execute(bytecode, gas_limit)?;

        let gas_used = gas_limit - self.gas_remaining;
        let result = self.memory[0..32].to_vec(); // Assume result is in the first 32 bytes of memory

        Ok((result, gas_used))
    }

    /// Performs addition operation
    fn op_add(&mut self) -> IcnResult<()> {
        if self.stack.len() < 2 {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in ADD".to_string()));
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(a.wrapping_add(b));
        self.gas_remaining = self.gas_remaining.saturating_sub(3);
        Ok(())
    }

    /// Performs subtraction operation
    fn op_sub(&mut self) -> IcnResult<()> {
        if self.stack.len() < 2 {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in SUB".to_string()));
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(a.wrapping_sub(b));
        self.gas_remaining = self.gas_remaining.saturating_sub(3);
        Ok(())
    }

    /// Performs multiplication operation
    fn op_mul(&mut self) -> IcnResult<()> {
        if self.stack.len() < 2 {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in MUL".to_string()));
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(a.wrapping_mul(b));
        self.gas_remaining = self.gas_remaining.saturating_sub(5);
        Ok(())
    }

    /// Performs division operation
    fn op_div(&mut self) -> IcnResult<()> {
        if self.stack.len() < 2 {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in DIV".to_string()));
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        if b == 0 {
            return Err(IcnError::VirtualMachine("Execution error: Division by zero".to_string()));
        }
        self.stack.push(a.wrapping_div(b));
        self.gas_remaining = self.gas_remaining.saturating_sub(5);
        Ok(())
    }

    /// Pushes a value onto the stack
    fn op_push(&mut self) -> IcnResult<()> {
        if self.program_counter >= self.memory.len() {
            return Err(IcnError::VirtualMachine("Memory access error: Out of bounds memory access".to_string()));
        }
        let value = self.memory[self.program_counter] as i64;
        self.program_counter += 1;
        self.stack.push(value);
        self.gas_remaining = self.gas_remaining.saturating_sub(3);
        Ok(())
    }

    /// Pops a value from the stack
    fn op_pop(&mut self) -> IcnResult<()> {
        if self.stack.is_empty() {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in POP".to_string()));
        }
        self.stack.pop();
        self.gas_remaining = self.gas_remaining.saturating_sub(2);
        Ok(())
    }

    /// Performs an unconditional jump
    fn op_jump(&mut self) -> IcnResult<()> {
        if self.stack.is_empty() {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in JUMP".to_string()));
        }
        let dest = self.stack.pop().unwrap() as usize;
        if dest >= self.memory.len() {
            return Err(IcnError::VirtualMachine("Execution error: Invalid jump destination".to_string()));
        }
        self.program_counter = dest;
        self.gas_remaining = self.gas_remaining.saturating_sub(8);
        Ok(())
    }

    /// Performs a conditional jump
    fn op_jumpi(&mut self) -> IcnResult<()> {
        if self.stack.len() < 2 {
            return Err(IcnError::VirtualMachine("Execution error: Stack underflow in JUMPI".to_string()));
        }
        let condition = self.stack.pop().unwrap();
        let dest = self.stack.pop().unwrap() as usize;
        if condition != 0 {
            if dest >= self.memory.len() {
                return Err(IcnError::VirtualMachine("Execution error: Invalid jump destination".to_string()));
            }
            self.program_counter = dest;
        }
        self.gas_remaining = self.gas_remaining.saturating_sub(10);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0x10, 5, 0x10, 3, 0x01, 0xFF]); // PUSH 5, PUSH 3, ADD, HALT
        assert!(vm.execute(bytecode, 1000).is_ok());
        assert_eq!(vm.stack, vec![8]);
    }

    #[test]
    fn test_out_of_gas() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0x10, 5, 0x10, 3, 0x01, 0xFF]); // PUSH 5, PUSH 3, ADD, HALT
        let result = vm.execute(bytecode, 5); // Not enough gas
        assert!(matches!(result, Err(IcnError::VirtualMachine(ref msg)) if msg == "Execution halted: Out of gas"));
    }

    #[test]
    fn test_invalid_opcode() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0xFF, 0xAA]); // HALT, Invalid opcode
        let result = vm.execute(bytecode, 1000);
        assert!(matches!(result, Err(IcnError::VirtualMachine(ref msg)) if msg.starts_with("Execution error: Invalid opcode")));
    }

    #[test]
    fn test_stack_underflow_add() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0x01, 0xFF]); // ADD (with empty stack), HALT
        let result = vm.execute(bytecode, 1000);
        assert!(matches!(result, Err(IcnError::VirtualMachine(ref msg)) if msg == "Execution error: Stack underflow in ADD"));
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = VirtualMachine::new();
        let bytecode = Bytecode::new(vec![0x10, 5, 0x10, 0, 0x04, 0xFF]); // PUSH 5, PUSH 0, DIV, HALT
        let result = vm.execute(bytecode, 1000);
        assert!(matches!(result, Err(IcnError::VirtualMachine(ref msg)) if msg == "Execution error: Division by zero"));
    }
}
