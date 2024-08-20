// icn_virtual_machine/src/bytecode.rs

/// The `Bytecode` struct represents the compiled code of a smart contract.
/// It contains a vector of bytes that the virtual machine can execute.
pub struct Bytecode {
    pub code: Vec<u8>,
}

impl Bytecode {
    /// Creates a new instance of `Bytecode`.
    pub fn new(code: Vec<u8>) -> Self {
        Bytecode { code }
    }
}
