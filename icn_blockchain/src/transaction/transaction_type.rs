/// Represents the different types of transactions supported by the blockchain
#[derive(Debug, Clone, PartialEq, Eq)] // Add necessary derives based on your project's requirements
pub enum TransactionType {
    Transfer,
    DeployContract,
    // Add more variants as needed
}
