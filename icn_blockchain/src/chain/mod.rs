// icn_blockchain/src/chain/mod.rs

use crate::block::Block;
use crate::transaction::Transaction;

pub struct Chain {
    pub blocks: Vec<Block>,
}

impl Chain {
    pub fn new() -> Self {
        let genesis_block = Block::new(
            0, 
            vec![], 
            String::from("0"), 
            String::from("genesis_proposer")
        );

        Chain {
            blocks: vec![genesis_block],
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>, previous_hash: String, proposer_id: String) {
        let previous_block = self.blocks.last().unwrap();
        let new_block = Block::new(
            previous_block.index + 1,
            transactions,
            previous_hash,
            proposer_id
        );
        self.blocks.push(new_block);
    }
}
