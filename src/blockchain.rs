use super::*;
use crate::block::*;
use crate::transaction::*;
use bincode::{deserialize, serialize};
use sled;
use std::collections::HashMap;

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
}

/// BlockchainIterator is used to iterate over blockchain blocks
struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    ///
    /// store the last hash with a key as "LAST", and the serialized block with a key as it's hash
    pub fn new() -> Result<Blockchain> {
        info!("Creating new blockchain");

        let db = sled::open("data/blocks")?;
        match db.get("LAST")? {
            Some(hash) => {
                info!("Found block database");
                let lasthash = String::from_utf8(hash.to_vec())?;
                Ok(Blockchain {
                    tip: lasthash.clone(),
                    db,
                })
            }
            None => {
                info!("Creating new block database");
                let block = Block::new_genesis_block();
                db.insert(block.get_hash(), serialize(&block)?)?;
                db.insert("LAST", block.get_hash().as_bytes())?;
                let bc = Blockchain {
                    tip: block.get_hash(),
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
        info!("add new block to the chain");

        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(data, String::from_utf8(lasthash.to_vec())?)?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();

        Ok(())
    }

    /// Iterator returns a BlockchainIterat
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip,
            bc: &self,
        }
    }

    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self, address: String) -> Vec<TXOutput> {
        let mut utxos = Vec::<TXOutput>::new();
        let unspend_TXs = self.find_unspent_transactions(address);
        for tx in unspend_TXs {
            for out in tx.vout {
                if out.can_be_unlock_with(address) {
                    utxos.push(out);
                }
            }
        }
        utxos
    }

    /// FindUnspentTransactions returns a list of transactions containing unspent outputs
    pub fn find_spendable_outputs(
        &self,
        address: String,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;
        let unspend_TXs = self.find_unspent_transactions(address);

        for tx in unspend_TXs {
            for index in 0..tx.vout.len() {
                if tx.vout[index].can_be_unlock_with(address) && accumulated < amount {
                    match unspent_outputs.get(&tx.id) {
                        Some(v) => v.push(index as i32),
                        None => {
                            unspent_outputs.insert(tx.id, vec![index as i32]);
                        }
                    }
                    accumulated += tx.vout[index].value;

                    if accumulated >= amount {
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }
        (accumulated, unspent_outputs)
    }

    /// FindUnspentTransactions returns a list of transactions containing unspent outputs
    fn find_unspent_transactions(&self, address: String) -> Vec<&Transaction> {
        let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut unspend_TXs: Vec<&Transaction> = Vec::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spent_TXOs.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    if tx.vout[index].can_be_unlock_with(address) {
                        unspend_TXs.push(tx)
                    }
                }

                if !tx.is_coinbase() {
                    for i in tx.vin {
                        if i.can_unlock_output_with(address) {
                            match spent_TXOs.get(&i.txid) {
                                Some(_) => spent_TXOs[&i.txid].push(i.vout),
                                None => {
                                    spent_TXOs.insert(i.txid, vec![i.vout]);
                                }
                            }
                        }
                    }
                }
            }
        }

        unspend_TXs
    }
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
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
