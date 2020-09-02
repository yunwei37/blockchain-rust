use super::*;
use crate::transaction::Transaction;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const TARGET_HEXS: usize = 4;

/// Block keeps block headers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    timestamp: u128,
    transaction: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
}

impl Block {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }

    pub fn get_transaction(&self) -> &Vec<Transaction> {
        &self.transaction
    }

    /// NewBlock creates and returns Block
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let mut block = Block {
            timestamp,
            data,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("Genesis Block"), String::new()).unwrap()
    }

    pub fn

    /// Run performs a proof-of-work
    fn run_proof_of_work(&mut self) -> Result<()> {
        info!("Mining the block containing \"{}\"\n", self.data);
        while !self.validate()? {
            self.nonce += 1;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.data.clone(),
            self.timestamp,
            TARGET_HEXS,
            self.nonce,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }

    /// Validate validates block's PoW
    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let mut vec1: Vec<u8> = Vec::new();
        vec1.resize(TARGET_HEXS, '0' as u8);
        Ok(&hasher.result_str()[0..TARGET_HEXS] == String::from_utf8(vec1)?)
    }
}
