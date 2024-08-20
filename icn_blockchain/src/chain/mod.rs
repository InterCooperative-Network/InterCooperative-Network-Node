use crate::block::Block;
use crate::transaction::Transaction;

pub struct Chain {
    pub blocks: Vec<Block>,
}

impl Chain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, vec![], String::from("0"));
        Chain {
            blocks: vec![genesis_block],
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> bool {
        let previous_block = self.blocks.last().unwrap();
        let new_block = Block::new(previous_block.index + 1, transactions, previous_block.hash.clone());

        if new_block.validate_transactions() {
            self.blocks.push(new_block);
            true
        } else {
            false
        }
    }
}
