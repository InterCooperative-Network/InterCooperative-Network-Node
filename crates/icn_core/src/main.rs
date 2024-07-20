use icn_core::{IcnNode, Config};
use icn_types::{IcnResult, IcnError, Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType};
use std::io::{self, Write};
use chrono::{Duration, Utc};
use log::{info, warn, error};

fn main() -> IcnResult<()> {
    env_logger::init();

    let config = Config::load("config.json").unwrap_or_else(|_| {
        warn!("Failed to load config.json, using default configuration");
        Config::default()
    });

    info!("Starting InterCooperative Network node...");
    let node = IcnNode::new(config)?;
    node.start()?;

    info!("Node started successfully. Type 'help' for available commands.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "help" => print_help(),
            "exit" => break,
            "transaction" => process_transaction(&node)?,
            "proposal" => create_proposal(&node)?,
            "balance" => check_balance(&node)?,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    info!("Stopping node...");
    node.stop()?;
    info!("Node stopped. Goodbye!");

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  help        - Show this help message");
    println!("  transaction - Create a new transaction");
    println!("  proposal    - Create a new proposal");
    println!("  balance     - Check account balance");
    println!("  exit        - Exit the application");
}

fn process_transaction(node: &IcnNode) -> IcnResult<()> {
    info!("Processing a new transaction");
    
    print!("From: ");
    io::stdout().flush().unwrap();
    let mut from = String::new();
    io::stdin().read_line(&mut from).unwrap();
    
    print!("To: ");
    io::stdout().flush().unwrap();
    let mut to = String::new();
    io::stdin().read_line(&mut to).unwrap();
    
    print!("Amount: ");
    io::stdout().flush().unwrap();
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    let amount: f64 = amount_str.trim().parse().map_err(|_| IcnError::InvalidInput("Invalid amount".to_string()))?;

    let transaction = Transaction {
        from: from.trim().to_string(),
        to: to.trim().to_string(),
        amount,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(transaction)?;
    info!("Transaction processed successfully");
    Ok(())
}

fn create_proposal(node: &IcnNode) -> IcnResult<()> {
    info!("Creating a new proposal");
    
    print!("Title: ");
    io::stdout().flush().unwrap();
    let mut title = String::new();
    io::stdin().read_line(&mut title).unwrap();
    
    print!("Description: ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description).unwrap();
    
    print!("Proposer: ");
    io::stdout().flush().unwrap();
    let mut proposer = String::new();
    io::stdin().read_line(&mut proposer).unwrap();

    let proposal = Proposal {
        id: String::new(),
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        proposer: proposer.trim().to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: icn_types::ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Economic,
        required_quorum: 0.51,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal)?;
    info!("Proposal created successfully. Proposal ID: {}", proposal_id);
    Ok(())
}

fn check_balance(node: &IcnNode) -> IcnResult<()> {
    info!("Checking account balance");
    
    print!("Address: ");
    io::stdout().flush().unwrap();
    let mut address = String::new();
    io::stdin().read_line(&mut address).unwrap();

    print!("Currency Type (BasicNeeds, Education, etc.): ");
    io::stdout().flush().unwrap();
    let mut currency_type_str = String::new();
    io::stdin().read_line(&mut currency_type_str).unwrap();

    let currency_type = match currency_type_str.trim() {
        "BasicNeeds" => CurrencyType::BasicNeeds,
        "Education" => CurrencyType::Education,
        // Add more currency types as needed
        _ => return Err(IcnError::InvalidInput("Invalid currency type".to_string())),
    };

    let balance = node.get_balance(address.trim(), &currency_type)?;
    println!("Balance: {}", balance);
    info!("Balance query successful");
    Ok(())
}