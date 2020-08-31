use super::*;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time::SystemTime;

/// Block keeps block headers
#[derive(Debug)]
pub struct Block {
    timestamp: u128,
    data: String,
    prev_block_hash: String,
    hash: String,
}

impl Block {
    /// SetHash calculates and sets block hash
    pub fn set_hash(&mut self) -> Result<()> {
        self.timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let content = (self.data.clone(), self.timestamp);
        let bytes = serialize(&content)?;
        let mut hasher = Sha256::new();
        hasher.input(&bytes[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    /// NewBlock creates and returns Block
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let mut block = Block {
            timestamp: 0,
            data,
            prev_block_hash,
            hash: String::new(),
        };
        block.set_hash()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("Genesis Block"), String::new()).unwrap()
    }
}
