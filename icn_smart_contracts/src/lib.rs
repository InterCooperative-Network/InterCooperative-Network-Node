// icn_smart_contracts/src/lib.rs

use std::collections::HashMap;

// Define our own error type for this crate
#[derive(Debug)]
pub enum SmartContractError {
    InvalidArguments(String),
    ContractNotFound(u32),
    KeyNotFound(String),
    UnknownFunction(String),
}

pub type SmartContractResult<T> = Result<T, SmartContractError>;

pub struct SmartContract {
    pub id: u32,
    pub code: String,
    pub state: HashMap<String, String>,
}

impl SmartContract {
    pub fn new(id: u32, code: &str) -> Self {
        SmartContract {
            id,
            code: code.to_string(),
            state: HashMap::new(),
        }
    }

    pub fn execute(&mut self, function: &str, args: Vec<String>) -> SmartContractResult<String> {
        match function {
            "set" => {
                if args.len() != 2 {
                    return Err(SmartContractError::InvalidArguments("'set' requires 2 arguments".to_string()));
                }
                self.state.insert(args[0].clone(), args[1].clone());
                Ok("Value set successfully".to_string())
            }
            "get" => {
                if args.len() != 1 {
                    return Err(SmartContractError::InvalidArguments("'get' requires 1 argument".to_string()));
                }
                self.state.get(&args[0])
                    .cloned()
                    .ok_or_else(|| SmartContractError::KeyNotFound(args[0].clone()))
            }
            _ => Err(SmartContractError::UnknownFunction(function.to_string())),
        }
    }
}

pub struct SmartContractEngine {
    contracts: HashMap<u32, SmartContract>,
}

impl SmartContractEngine {
    pub fn new() -> Self {
        SmartContractEngine {
            contracts: HashMap::new(),
        }
    }

    pub fn deploy_contract(&mut self, code: &str) -> SmartContractResult<u32> {
        let id = self.contracts.len() as u32 + 1;
        let contract = SmartContract::new(id, code);
        self.contracts.insert(id, contract);
        Ok(id)
    }

    pub fn call_contract(&mut self, id: u32, function: &str, args: Vec<String>) -> SmartContractResult<String> {
        self.contracts.get_mut(&id)
            .ok_or_else(|| SmartContractError::ContractNotFound(id))?
            .execute(function, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_contract_execution() {
        let mut contract = SmartContract::new(1, "sample code");
        
        let result = contract.execute("set", vec!["key".to_string(), "value".to_string()]);
        assert!(result.is_ok());

        let result = contract.execute("get", vec!["key".to_string()]);
        assert_eq!(result.unwrap(), "value");

        let result = contract.execute("unknown", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_smart_contract_engine() {
        let mut engine = SmartContractEngine::new();

        let contract_id = engine.deploy_contract("sample code").unwrap();
        assert_eq!(contract_id, 1);

        let result = engine.call_contract(contract_id, "set", vec!["key".to_string(), "value".to_string()]);
        assert!(result.is_ok());

        let result = engine.call_contract(contract_id, "get", vec!["key".to_string()]);
        assert_eq!(result.unwrap(), "value");

        let result = engine.call_contract(999, "get", vec!["key".to_string()]);
        assert!(result.is_err());
    }
}