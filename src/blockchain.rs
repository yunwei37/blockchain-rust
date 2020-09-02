use super::*;
use crate::block::*;
use bincode::{deserialize, serialize};
use sled;

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    current_hash: String,
    db: sled::Db,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    ///
    /// store the last hash with a key as "LAST", and the serialized block with a key as it's hash
    pub fn new() -> Result<Blockchain> {
        let db = sled::open("/tmp/blocks")?;

        match db.get("LAST")? {
            Some(hash) => {
                let lasthash = String::from_utf8(hash.to_vec())?;
                Ok(Blockchain {
                    tip: lasthash.clone(),
                    current_hash: lasthash,
                    db,
                })
            }
            None => {
                let block = Block::new_genesis_block();
                db.insert(block.get_hash(), serialize(&block)?)?;
                db.insert("LAST", block.get_hash().as_bytes())?;
                let bc = Blockchain {
                    tip: block.get_hash(),
                    current_hash: block.get_hash(),
                    db,
                };
                bc.db.flush()?;
                Ok(bc)
            }
        }
    }

    /// AddBlock saves provided data as a block in the blockchain
    ///
    /// Save the block into the database
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(data, String::from_utf8(lasthash.to_vec())?)?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        self.current_hash = newblock.get_hash();

        Ok(())
    }
}

impl Iterator for Blockchain {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}
