// Filename: icn_consensus/src/consensus.rs

use icn_shared::{Block, IcnResult};

/// The `Consensus` trait defines the interface for consensus mechanisms
/// within the InterCooperative Network (ICN) blockchain system.
///
/// This trait encapsulates the core functionalities required for maintaining
/// consensus across the network. By implementing this trait, different
/// consensus algorithms can be used interchangeably within the blockchain,
/// allowing for flexibility and experimentation with various consensus models.
///
/// The trait requires implementation of methods for block validation,
/// proposer selection, peer eligibility, and state updates. These methods
/// collectively ensure that all nodes in the network can agree on the
/// current state of the blockchain and the process for adding new blocks.
///
/// Implementors of this trait must also derive or implement `Clone`, `Send`,
/// and `Sync` to ensure thread-safety and ease of use across the system.
pub trait Consensus: Clone + Send + Sync {
    /// Validates a block according to the consensus rules.
    ///
    /// This method is crucial for maintaining the integrity of the blockchain.
    /// It should check all relevant aspects of the block, including its structure,
    /// transactions, and adherence to network rules.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block that needs to be validated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid,
    ///   `Ok(false)` if the block is invalid but not due to an error,
    ///   or an `IcnError` if validation fails due to an error condition.
    fn validate(&mut self, block: &Block) -> IcnResult<bool>;

    /// Selects a proposer for the next block based on the consensus mechanism's rules.
    ///
    /// This method implements the logic for choosing which node in the network
    /// has the right to propose the next block. The selection process can vary
    /// widely between different consensus algorithms (e.g., round-robin, stake-weighted
    /// random selection, etc.).
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the ID of the selected proposer as a `String`,
    ///   or an `IcnError` if the selection process fails.
    fn select_proposer(&mut self) -> IcnResult<String>;

    /// Retrieves the list of eligible peers for proposer selection.
    ///
    /// This method should return a list of peer IDs that are currently eligible
    /// to be selected as block proposers. The criteria for eligibility can vary
    /// based on the specific consensus algorithm (e.g., minimum stake, uptime
    /// requirements, etc.).
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector containing the IDs of eligible peers.
    fn get_eligible_peers(&self) -> Vec<String>;

    /// Updates the internal state of the consensus mechanism based on the current blockchain state.
    ///
    /// This method is called after a new block is added to the chain, allowing
    /// the consensus mechanism to update its internal state accordingly. This
    /// could involve updating stake information, resetting timers, or any other
    /// state changes required by the specific consensus algorithm.
    ///
    /// # Arguments
    ///
    /// * `latest_block` - A reference to the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the state update is successful,
    ///   or an `IcnError` if the update fails.
    fn update_state(&mut self, latest_block: &Block) -> IcnResult<()>;

    /// Initializes the consensus mechanism with the current blockchain state.
    ///
    /// This method should be called when the node starts up or when it needs
    /// to resynchronize with the network. It allows the consensus mechanism
    /// to initialize its internal state based on the current blockchain.
    ///
    /// # Arguments
    ///
    /// * `latest_block` - A reference to the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if initialization is successful,
    ///   or an `IcnError` if it fails.
    fn initialize(&mut self, latest_block: &Block) -> IcnResult<()>;

    /// Handles network events that may affect the consensus state.
    ///
    /// This method allows the consensus mechanism to react to various network
    /// events, such as peer connections/disconnections, network partitions, etc.
    ///
    /// # Arguments
    ///
    /// * `event` - An enum representing different types of network events.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the event is handled successfully,
    ///   or an `IcnError` if handling fails.
    fn handle_network_event(&mut self, event: NetworkEvent) -> IcnResult<()>;
}

/// Represents various network events that may affect the consensus state.
///
/// This enum allows the consensus mechanism to be notified of and react to
/// important network-level events that could impact its operation or decision-making.
pub enum NetworkEvent {
    /// A new peer has connected to the network.
    PeerConnected(String),
    /// An existing peer has disconnected from the network.
    PeerDisconnected(String),
    /// A network partition has been detected.
    NetworkPartitionDetected,
    /// The network has been reunified after a partition.
    NetworkReunified,
    /// A significant change in network conditions has been detected.
    NetworkConditionChanged(NetworkCondition),
}

/// Represents the current condition or state of the network.
///
/// This enum allows for reporting of various network states that might
/// affect the consensus mechanism's operation or efficiency.
pub enum NetworkCondition {
    /// The network is operating normally.
    Normal,
    /// The network is experiencing high latency.
    HighLatency,
    /// The network is congested.
    Congested,
    /// The network is unstable.
    Unstable,
}