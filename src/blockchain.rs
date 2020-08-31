use super::*;
use crate::block::*;

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![Block::new_genesis_block()],
        }
    }

    /// AddBlock saves provided data as a block in the blockchain
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let prev = self.blocks.last().unwrap();
        let newblock = Block::new_block(data, prev.get_hash())?;
        self.blocks.push(newblock);
        Ok(())
    }
}
