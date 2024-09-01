// File: icn_virtual_machine/src/bytecode.rs

/// Represents an opcode in the ICN Virtual Machine's instruction set
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,

    // Logic operations
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,

    // Data manipulation
    Push,
    Pop,
    Load,
    Store,

    // Control flow
    Jump,
    JumpIfTrue,
    JumpIfFalse,

    // Blockchain interaction
    GetState,
    SetState,
    CallContract,

    // Other
    Halt,
    // ... more opcodes as needed
}

/// Represents the compiled bytecode of a smart contract
pub struct Bytecode {
    /// The raw bytecode instructions
    pub code: Vec<u8>,
}

impl Bytecode {
    /// Creates a new `Bytecode` instance
    pub fn new(code: Vec<u8>) -> Self {
        Bytecode { code }
    }

    /// Decodes the bytecode into a sequence of `Opcode` and operands
    pub fn decode(&self) -> Vec<(Opcode, Vec<u8>)> {
        let mut instructions = Vec::new();
        let mut pc = 0;

        while pc < self.code.len() {
            let opcode = Opcode::from(self.code[pc]);
            pc += 1;

            let operand_size = match opcode {
                Opcode::Push => {
                    // Assuming Push takes a single byte operand for simplicity
                    // You might need to adjust this based on your actual operand sizes
                    1
                }
                // Add more cases for other opcodes with operands
                _ => 0, 
            };

            let operands = self.code[pc..pc + operand_size].to_vec();
            pc += operand_size;

            instructions.push((opcode, operands));
        }

        instructions
    }
}

impl From<u8> for Opcode {
    fn from(byte: u8) -> Self {
        match byte {
            0x01 => Opcode::Add,
            0x02 => Opcode::Subtract,
            // ... other opcode mappings
            _ => panic!("Invalid opcode: {}", byte), // Handle invalid opcodes gracefully in your actual implementation
        }
    }
}