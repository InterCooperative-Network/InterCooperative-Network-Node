// icn_blockchain/src/chain/mod.rs

use crate::block::Block;
use std::collections::VecDeque;

pub struct Chain {
    blocks: VecDeque<Block>,
}

impl Chain {
    pub fn new() -> Self {
        let genesis_block = create_genesis_block();
        let mut blocks = VecDeque::new();
        blocks.push_back(genesis_block);
        Chain { blocks }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push_back(block);
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.back()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

fn create_genesis_block() -> Block {
    let transactions = vec![];
    Block::new(0, 0, transactions, String::from("0"), String::from("genesis_hash"), String::from("genesis_proposer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_creation() {
        let chain = Chain::new();
        assert_eq!(chain.len(), 1);
        assert!(chain.get_latest_block().is_some());
    }

    #[test]
    fn test_add_block() {
        let mut chain = Chain::new();
        let new_block = Block::new(1, 0, vec![], String::from("prev_hash"), String::from("new_hash"), String::from("proposer"));
        chain.add_block(new_block);
        assert_eq!(chain.len(), 2);
    }
}