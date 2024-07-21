use icn_common::{IcnResult, IcnError, CurrencyType};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
    pub creation_date: DateTime<Utc>,
    pub last_issuance: DateTime<Utc>,
    pub issuance_rate: f64,
}

impl Currency {
    pub fn new(currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) -> Self {
        let now = Utc::now();
        Currency {
            currency_type,
            total_supply: initial_supply,
            creation_date: now,
            last_issuance: now,
            issuance_rate,
        }
    }

    pub fn mint(&mut self, amount: f64) -> IcnResult<()> {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
        Ok(())
    }

    pub fn burn(&mut self, amount: f64) -> IcnResult<()> {
        if amount > self.total_supply {
            return Err(IcnError::Currency("Insufficient supply to burn".to_string()));
        }
        self.total_supply -= amount;
        Ok(())
    }
}

pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: HashMap::new(),
            balances: HashMap::new(),
        };
        
        system.add_currency(CurrencyType::BasicNeeds, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Environmental, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Community, 250_000.0, 0.003);
        system.add_currency(CurrencyType::Volunteer, 100_000.0, 0.002);

        system
    }

    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    pub fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        self.balances
            .get(address)
            .and_then(|balances| balances.get(currency_type))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn update_balance(&mut self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let balance = self.balances
            .entry(address.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .or_insert(0.0);
        *balance += amount;
        if *balance < 0.0 {
            return Err(IcnError::Currency("Insufficient balance".to_string()));
        }
        Ok(())
    }

    pub fn transfer(&mut self, from: &str, to: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.update_balance(from, currency_type, -amount)?;
        self.update_balance(to, currency_type, amount)?;
        Ok(())
    }

    pub fn process_transaction(&mut self, from: &str, to: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.transfer(from, to, currency_type, amount)
    }

    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> IcnResult<()> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(IcnError::Currency(format!("Currency '{}' already exists", name)));
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    pub fn adaptive_issuance(&mut self) -> IcnResult<()> {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount)?;
        }
        Ok(())
    }

    pub fn mint(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", currency_type)))?;
        currency.mint(amount)
    }

    pub fn burn(&mut self, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let currency = self.currencies.get_mut(currency_type)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", currency_type)))?;
        currency.burn(amount)
    }

    pub fn get_exchange_rate(&self, from: &CurrencyType, to: &CurrencyType) -> IcnResult<f64> {
        // This is a placeholder implementation. In a real-world scenario, 
        // exchange rates would be determined by market forces or a more complex algorithm.
        if from == to {
            return Ok(1.0);
        }
        
        let from_currency = self.currencies.get(from)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", from)))?;
        let to_currency = self.currencies.get(to)
            .ok_or_else(|| IcnError::Currency(format!("Currency {:?} not found", to)))?;

        Ok(from_currency.total_supply / to_currency.total_supply)
    }

    pub fn convert_currency(&mut self, from: &CurrencyType, to: &CurrencyType, amount: f64) -> IcnResult<f64> {
        let exchange_rate = self.get_exchange_rate(from, to)?;
        Ok(amount * exchange_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        
        // Test balance operations
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 0.0);
        system.update_balance("Alice", &CurrencyType::BasicNeeds, 100.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 100.0);

        // Test transfer
        system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 50.0).unwrap();
        assert_eq!(system.get_balance("Alice", &CurrencyType::BasicNeeds), 50.0);
        assert_eq!(system.get_balance("Bob", &CurrencyType::BasicNeeds), 50.0);

        // Test custom currency creation
        system.create_custom_currency("LocalCoin".to_string(), 10_000.0, 0.005).unwrap();
        let local_coin = CurrencyType::Custom("LocalCoin".to_string());
        assert!(system.currencies.contains_key(&local_coin));

        // Test adaptive issuance
        let initial_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        system.adaptive_issuance().unwrap();
        assert!(system.currencies[&CurrencyType::BasicNeeds].total_supply > initial_supply);

        // Test mint and burn
        let mint_amount = 1000.0;
        system.mint(&CurrencyType::BasicNeeds, mint_amount).unwrap();
        let new_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        assert_eq!(new_supply, initial_supply + mint_amount);

        system.burn(&CurrencyType::BasicNeeds, 500.0).unwrap();
        let final_supply = system.currencies[&CurrencyType::BasicNeeds].total_supply;
        assert_eq!(final_supply, new_supply - 500.0);

        // Test insufficient balance error
        assert!(system.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 1000.0).is_err());

        // Test currency conversion
        let exchange_rate = system.get_exchange_rate(&CurrencyType::BasicNeeds, &CurrencyType::Education).unwrap();
        assert!(exchange_rate > 0.0);

        let converted_amount = system.convert_currency(&CurrencyType::BasicNeeds, &CurrencyType::Education, 100.0).unwrap();
        assert!(converted_amount > 0.0);
    }

    #[test]
    fn test_currency_operations() {
        let mut currency = Currency::new(CurrencyType::BasicNeeds, 1000.0, 0.01);

        // Test minting
        currency.mint(100.0).unwrap();
        assert_eq!(currency.total_supply, 1100.0);

        // Test burning
        currency.burn(50.0).unwrap();
        assert_eq!(currency.total_supply, 1050.0);

        // Test burning more than available
        assert!(currency.burn(2000.0).is_err());
    }

    #[test]
    fn test_custom_currency() {
        let mut system = CurrencySystem::new();

        // Create a custom currency
        system.create_custom_currency("GreenCoin".to_string(), 5000.0, 0.02).unwrap();

        // Verify the custom currency was created
        let green_coin = CurrencyType::Custom("GreenCoin".to_string());
        assert!(system.currencies.contains_key(&green_coin));

        // Test operations with custom currency
        system.update_balance("Alice", &green_coin, 100.0).unwrap();
        assert_eq!(system.get_balance("Alice", &green_coin), 100.0);

        system.transfer("Alice", "Bob", &green_coin, 50.0).unwrap();
        assert_eq!(system.get_balance("Alice", &green_coin), 50.0);
        assert_eq!(system.get_balance("Bob", &green_coin), 50.0);

        // Test creating a duplicate custom currency
        assert!(system.create_custom_currency("GreenCoin".to_string(), 1000.0, 0.01).is_err());
    }

    #[test]
    fn test_exchange_rates() {
        let mut system = CurrencySystem::new();

        // Test exchange rate between two existing currencies
        let rate = system.get_exchange_rate(&CurrencyType::BasicNeeds, &CurrencyType::Education).unwrap();
        assert!(rate > 0.0);

        // Test exchange rate with custom currency
        system.create_custom_currency("TechCoin".to_string(), 100_000.0, 0.03).unwrap();
        let tech_coin = CurrencyType::Custom("TechCoin".to_string());
        let rate = system.get_exchange_rate(&CurrencyType::BasicNeeds, &tech_coin).unwrap();
        assert!(rate > 0.0);

        // Test currency conversion
        let amount = 100.0;
        let converted = system.convert_currency(&CurrencyType::BasicNeeds, &CurrencyType::Education, amount).unwrap();
        assert!(converted > 0.0);

        // Test exchange rate with non-existent currency
        let non_existent = CurrencyType::Custom("NonExistent".to_string());
        assert!(system.get_exchange_rate(&CurrencyType::BasicNeeds, &non_existent).is_err());
    }
}