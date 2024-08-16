// File: crates/icn_vm/src/smart_contract.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmartContract {
    pub id: String,
    pub code: String,
    pub state: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

pub struct SmartContractExecutor {
    contracts: Arc<RwLock<HashMap<String, SmartContract>>>,
}

impl SmartContractExecutor {
    pub fn new() -> Self {
        SmartContractExecutor {
            contracts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn deploy_contract(&self, id: String, code: String) -> IcnResult<()> {
        let mut contracts = self.contracts.write().map_err(|_| IcnError::SmartContract("Failed to acquire write lock".into()))?;
        
        if contracts.contains_key(&id) {
            return Err(IcnError::SmartContract(format!("Contract with id {} already exists", id)));
        }

        let contract = SmartContract {
            id: id.clone(),
            code,
            state: HashMap::new(),
        };

        contracts.insert(id, contract);
        Ok(())
    }

    pub fn execute_contract(&self, id: &str, function: &str, args: Vec<Value>) -> IcnResult<Value> {
        let contracts = self.contracts.read().map_err(|_| IcnError::SmartContract("Failed to acquire read lock".into()))?;
        
        let contract = contracts.get(id)
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with id {} not found", id)))?;

        // In a real implementation, you would parse and execute the contract code here.
        // For this example, we'll simulate a simple token transfer function.
        match function {
            "transfer" => self.execute_transfer(contract, args),
            _ => Err(IcnError::SmartContract(format!("Unknown function: {}", function))),
        }
    }

    fn execute_transfer(&self, contract: &SmartContract, args: Vec<Value>) -> IcnResult<Value> {
        if args.len() != 3 {
            return Err(IcnError::SmartContract("transfer function requires 3 arguments: from, to, and amount".into()));
        }

        let from = match &args[0] {
            Value::String(s) => s,
            _ => return Err(IcnError::SmartContract("'from' argument must be a string".into())),
        };

        let to = match &args[1] {
            Value::String(s) => s,
            _ => return Err(IcnError::SmartContract("'to' argument must be a string".into())),
        };

        let amount = match &args[2] {
            Value::Int(n) => *n as f64,
            Value::Float(n) => *n,
            _ => return Err(IcnError::SmartContract("'amount' argument must be a number".into())),
        };

        let mut contracts = self.contracts.write().map_err(|_| IcnError::SmartContract("Failed to acquire write lock".into()))?;
        let contract = contracts.get_mut(&contract.id).ok_or_else(|| IcnError::SmartContract("Contract not found".into()))?;

        let balances = contract.state.entry("balances".to_string())
            .or_insert_with(|| Value::Map(HashMap::new()));

        if let Value::Map(ref mut balance_map) = balances {
            let from_balance = balance_map.entry(from.to_string())
                .or_insert(Value::Float(0.0));
            
            let to_balance = balance_map.entry(to.to_string())
                .or_insert(Value::Float(0.0));

            if let (Value::Float(from_amount), Value::Float(to_amount)) = (from_balance, to_balance) {
                if *from_amount < amount {
                    return Err(IcnError::SmartContract("Insufficient balance for transfer".into()));
                }

                *from_amount -= amount;
                *to_amount += amount;

                Ok(Value::Bool(true))
            } else {
                Err(IcnError::SmartContract("Invalid balance type".into()))
            }
        } else {
            Err(IcnError::SmartContract("Invalid state structure".into()))
        }
    }

    pub fn get_contract_state(&self, id: &str) -> IcnResult<HashMap<String, Value>> {
        let contracts = self.contracts.read().map_err(|_| IcnError::SmartContract("Failed to acquire read lock".into()))?;
        contracts.get(id)
            .map(|contract| contract.state.clone())
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with id {} not found", id)))
    }

    pub fn update_contract_state(&self, id: &str, key: String, value: Value) -> IcnResult<()> {
        let mut contracts = self.contracts.write().map_err(|_| IcnError::SmartContract("Failed to acquire write lock".into()))?;
        let contract = contracts.get_mut(id)
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with id {} not found", id)))?;

        contract.state.insert(key, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_contract_deployment_and_execution() {
        let executor = SmartContractExecutor::new();

        // Deploy a simple token contract
        let contract_id = "token_contract".to_string();
        let contract_code = r#"
            function transfer(from, to, amount) {
                // Transfer logic is implemented in the executor
            }
        "#.to_string();

        executor.deploy_contract(contract_id.clone(), contract_code).unwrap();

        // Initialize some balances
        executor.update_contract_state(&contract_id, "balances".to_string(), Value::Map(HashMap::new())).unwrap();
        let mut initial_balances = HashMap::new();
        initial_balances.insert("Alice".to_string(), Value::Float(100.0));
        initial_balances.insert("Bob".to_string(), Value::Float(50.0));
        executor.update_contract_state(&contract_id, "balances".to_string(), Value::Map(initial_balances)).unwrap();

        // Execute a transfer
        let result = executor.execute_contract(
            &contract_id,
            "transfer",
            vec![
                Value::String("Alice".to_string()),
                Value::String("Bob".to_string()),
                Value::Float(30.0),
            ],
        ).unwrap();

        assert_eq!(result, Value::Bool(true));

        // Check the updated balances
        let state = executor.get_contract_state(&contract_id).unwrap();
        if let Value::Map(balances) = &state["balances"] {
            assert_eq!(balances["Alice"], Value::Float(70.0));
            assert_eq!(balances["Bob"], Value::Float(80.0));
        } else {
            panic!("Invalid state structure");
        }

        // Test insufficient balance
        let result = executor.execute_contract(
            &contract_id,
            "transfer",
            vec![
                Value::String("Alice".to_string()),
                Value::String("Bob".to_string()),
                Value::Float(100.0),
            ],
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            IcnError::SmartContract("Insufficient balance for transfer".into()).to_string()
        );
    }
}