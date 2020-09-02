use super::*;
use crate::blockchain::*;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const subsidy: i32 = 10;

/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub script_sig: String,
}

/// TXOutput represents a transaction output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub script_pub_key: String,
}

/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    /// NewUTXOTransaction creates a new transaction
    pub fn new_UTXO(from: String, to: String, amount: i32, bc: &Blockchain) -> Result<Transaction> {
        let mut vin = Vec::new();
        let mut acc_v = (0, HashMap::new());
        acc_v = bc.find_spendable_outputs(from, amount);

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0,
                    vout: out,
                    script_sig: from,
                };
                vin.push(input);
            }
        }

        let vout = vec![TXOutput {
            value: amount,
            script_pub_key: to,
        }];
        if acc_v.0 > amount {
            vout.push(TXOutput {
                value: acc_v.0 - amount,
                script_pub_key: from,
            })
        }

        let tx = Transaction {
            id: String::new(),
            vin,
            vout,
        };
        tx.set_id()?;
        Ok(tx)
    }

    /// NewCoinbaseTX creates a new coinbase transaction
    pub fn new_coinbase(to: String, data: String) -> Result<Transaction> {
        if data == String::from("") {
            data += &format!("Reward to '{}'", to);
        }
        let tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                script_sig: data,
            }],
            vout: vec![TXOutput {
                value: subsidy,
                script_pub_key: to,
            }],
        };
        tx.set_id()?;
        Ok(tx)
    }

    /// SetID sets ID of a transaction
    fn set_id(&mut self) -> Result<()> {
        let mut hasher = Sha256::new();
        let data = serialize(self)?;
        hasher.input(&data);
        self.id = hasher.result_str();
        Ok(())
    }

    /// IsCoinbase checks whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
}

impl TXInput {
    /// CanUnlockOutputWith checks whether the address initiated the transaction
    pub fn can_unlock_output_with(&self, unlockingData: String) -> bool {
        self.script_sig == unlockingData
    }
}

impl TXOutput {
    /// CanBeUnlockedWith checks if the output can be unlocked with the provided data
    pub fn can_be_unlock_with(&self, unlockingData: String) -> bool {
        self.script_pub_key == unlockingData
    }
}
